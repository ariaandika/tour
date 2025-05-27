//! [`ExprParser`] implementation via syn
//!
//! see [`SynParser`]
use crate::{spec, syntax::*, TemplDisplay};
use quote::{quote, ToTokens};
use syn::*;
use tour_core::{Delimiter, ParseError, Visitor, Result};

macro_rules! error {
    ($($tt:tt)*) => {
        return Err(ParseError::Generic(format!($($tt)*)))
    };
}

/// layout information from a template
pub struct LayoutInfo {
    pub source: String,
    pub is_root: bool,
}

pub enum Scope {
    Root { stmts: Vec<Stmt> },
    If {
        templ: IfTempl,
        stmts: Vec<Stmt>,
        else_branch: Option<(Token![else],Box<Scope>)>
    },
    For {
        templ: ForTempl,
        stmts: Vec<Stmt>,
        else_branch: Option<(Token![else],Box<Scope>)>
    },
}

impl Scope {
    fn stack(&mut self) -> &mut Vec<Stmt> {
        match self {
            Scope::Root { stmts } => stmts,
            Scope::For { else_branch: Some(branch), .. } => branch.1.stack(),
            Scope::For { stmts, .. } => stmts,
            Scope::If { else_branch: Some(branch), .. } => branch.1.stack(),
            Scope::If { stmts, .. } => stmts,
        }
    }
}

pub enum Reload {
    Debug,
    Always,
    Never,
    Expr(Expr),
}

impl Reload {
    pub fn as_bool(&self) -> Result<bool,&Expr> {
        match self {
            Reload::Debug => Ok(cfg!(debug_assertions)),
            Reload::Always => Ok(true),
            Reload::Never => Ok(false),
            Reload::Expr(expr) => Err(expr),
        }
    }
}

/// [`Visitor`] implementation via syn
pub struct SynParser {
    pub layout: Option<LayoutInfo>,
    pub root: Vec<Stmt>,
    pub scopes: Vec<Scope>,
    pub static_len: usize,
    pub reload: Reload,
    pub statics: Vec<String>,
}

impl SynParser {
    pub fn new(reload: Reload) -> Self {
        Self {
            layout: None,
            root: vec![],
            scopes: vec![],
            static_len: 0,
            reload,
            statics: vec![],
        }
    }

    fn push_stack(&mut self, stmt: Stmt) {
        match self.scopes.last_mut() {
            Some(ok) => ok.stack(),
            None => &mut self.root,
        }.push(stmt);
    }
}

impl Visitor<'_> for SynParser {
    fn visit_static(&mut self, source: &str) -> Result<()> {
        let idx = Index::from(self.static_len);
        let src = match self.reload.as_bool() {
            Ok(cond) => if cond { quote! {&sources[#idx]} } else { quote! {#source} },
            Err(expr) => quote! { if #expr { &sources[#idx] } else { #source } },
        };

        self.static_len += 1;
        self.push_stack(syn::parse_quote!( #TemplDisplay::display(#src, writer)?; ));
        self.statics.push(source.to_owned());

        Ok(())
    }

    fn visit_expr(&mut self, source: &str, delim: Delimiter) -> Result<()> {
        let ok = match syn::parse_str(source) {
            Ok(ok) => ok,
            Err(err) => error!("failed to parse expr: {err}"),
        };

        match ok {
            ExprTempl::Layout(LayoutTempl { root_token, source, .. }) |
            ExprTempl::Extends(ExtendsTempl { root_token, source, .. }) => {
                if self.layout.is_some() {
                    error!("cannot have 2 `extends` or `layout`")
                }
                self.layout.replace(LayoutInfo {
                    source: source.value(),
                    is_root: root_token.is_some(),
                });
            }
            ExprTempl::Yield(_yield) => {
                self.push_stack(syn::parse_quote! {
                    #TemplDisplay::display(&layout_inner, &mut *writer)?;
                });
            }
            ExprTempl::Expr(expr) => {
                let display = spec::display(delim, expr);
                let writer = spec::writer(delim);
                self.push_stack(syn::parse_quote! {
                    #TemplDisplay::display(#display, #writer)?;
                });
            }
            ExprTempl::If(templ) => {
                self.scopes.push(Scope::If { templ, stmts: vec![], else_branch: None, });
            }
            ExprTempl::For(templ) => {
                self.scopes.push(Scope::For { templ, stmts: vec![], else_branch: None });
            }

            ExprTempl::Else(ElseTempl { else_token, elif_branch }) => {
                type ElseBranch = Option<(Token![else], Box<Scope>)>;

                fn take_latest_else_branch(else_branch: &mut ElseBranch) -> Result<&mut ElseBranch> {
                    match else_branch {
                        Some((_, branch)) => match &mut **branch {
                            // previously `else if`, we can fill more else branches
                            Scope::If { else_branch, .. } => take_latest_else_branch(else_branch),
                            Scope::Root { .. } => error!("cannot have 2 `else`"),
                            _ => panic!("`else` scope should only contain Root or If"),
                        },
                        // if current else branch is not filled, meancs its the latest
                        None => Ok(else_branch),
                    }
                }

                match self.scopes.pop() {
                    // else in if scope
                    Some(Scope::If { templ, stmts, mut else_branch, }) => {
                        take_latest_else_branch(&mut else_branch)?.replace((
                            else_token,
                            match elif_branch {
                                Some((if_token, cond)) => Scope::If {
                                    templ: IfTempl { if_token, cond },
                                    stmts: vec![],
                                    else_branch: None,
                                },
                                None => Scope::Root { stmts: vec![] },
                            }
                            .into(),
                        ));

                        self.scopes.push(Scope::If { templ, stmts, else_branch });
                    }
                    // else in for scope
                    Some(Scope::For { templ, stmts, mut else_branch, }) => {
                        if else_branch.is_some() {
                            error!("cannot have 2 `else` in `for` scope")
                        }
                        else_branch.replace((else_token, Scope::Root { stmts: vec![] }.into()));
                        self.scopes.push(Scope::For { templ, stmts, else_branch });
                    }
                    Some(scope) => error!("cannot close `else` in `{scope}` scope"),
                    None => error!("cannot close `else` in toplevel"),
                };
            }


            ExprTempl::EndIf(_endif) => {
                let (IfTempl { if_token, cond },stmts,else_branch) = match self.scopes.pop() {
                    Some(Scope::If { templ, stmts, else_branch, }) => (templ,stmts,else_branch),
                    Some(scope) => error!("cannot close `endif` in `{scope}` scope"),
                    None => error!("cannot close `endif` in toplevel"),
                };

                let else_branch = else_branch
                    .map(else_branch_expr)
                    .map(|(el, expr)| quote! {#el #expr});

                self.push_stack(syn::parse_quote! {
                    #if_token #cond {
                        #(#stmts)*
                    }
                    #else_branch
                });
            }
            ExprTempl::EndFor(_endfor) => {
                let (ForTempl { for_token, pat, in_token, expr },stmts,else_branch) = match self.scopes.pop() {
                    Some(Scope::For { templ, stmts, else_branch }) => (templ,stmts,else_branch),
                    Some(scope) => error!("cannot close `else` in `{scope}` scope"),
                    None => error!("cannot close `else` in toplevel"),
                };

                let for_expr = if else_branch.is_some() {
                    quote! { __for_expr }
                } else {
                    expr.to_token_stream()
                };

                if let Some((_,body)) = else_branch.map(else_branch_expr) {
                    self.push_stack(syn::parse_quote!(let __for_expr = #expr;));
                    self.push_stack(syn::parse_quote!(
                        if ExactSizeIterator::len(&IntoIterator::into_iter(__for_expr)) == 0 #body
                    ));
                }

                self.push_stack(syn::parse_quote!(#for_token #pat #in_token #for_expr { #(#stmts)* }));
            }

            ExprTempl::Use(UseTempl { use_token, path }) => {
                self.push_stack(syn::parse_quote! {
                    #use_token #path;
                });
            }
        }

        Ok(())
    }

    fn finish(mut self) -> Result<Self> {
        if let Some(scope) = self.scopes.pop() {
            error!("unclosed `{scope}` scope")
        }

        // Ok(SynOutput {
        //     layout: self.layout,
        //     stmts: self.root,
        //     reload: self.reload,
        // })
        Ok(self)
    }
}

fn else_branch_expr((el,scope):(Token![else],Box<Scope>)) -> (Token![else], Box<Expr>) {
    match *scope {
        Scope::Root { stmts } => (el,syn::parse_quote!({ #(#stmts)* })),
        Scope::If {
            templ: IfTempl { if_token, cond },
            stmts,
            else_branch
        } => {
            let else_branch = else_branch
                .map(else_branch_expr)
                .map(|(el,expr)|quote! {#el #expr});
            (el, syn::parse_quote!(#if_token #cond { #(#stmts)* } #else_branch))
        },
        _ => panic!("else scope should only contain Root or If"),
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Root { .. } => f.write_str("root"),
            Scope::If { .. } => f.write_str("if"),
            Scope::For { .. } => f.write_str("for"),
        }
    }
}

