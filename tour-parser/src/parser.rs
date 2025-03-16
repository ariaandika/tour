//! template source code parser
//!
//! use [`Parser`] to parse source code which output a [`Template`]
//!
//! template then can be generated to static source code at compile time
//! or static content at runtime
use crate::token::*;
use quote::quote;
use syn::*;

macro_rules! error {
    ($($tt:tt)*) => {
        return Err(Error::Generic(format!($($tt)*)))
    };
}

/// parse output
///
/// template then can be generated to static source code at compile time
/// or static content at runtime
pub struct Template {
    pub layout: Option<LayoutTempl>,
    /// rust statements which render and logic
    pub stmts: Vec<Stmt>,
    /// static contents
    pub statics: Vec<String>,
}

enum Scope {
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

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Root { .. } => f.write_str("root"),
            Scope::If { .. } => f.write_str("if"),
            Scope::For { .. } => f.write_str("for"),
        }
    }
}

enum ParseState {
    Static { start: usize },
    Expr { start: usize },
    OpenExpr { start: usize, brace: usize, },
    CloseExpr { start: usize, brace: usize, },
}

/// template source code parser
pub struct Parser<'a> {
    source: &'a [u8],

    // parser states
    index: usize,
    state: ParseState,
    scopes: Vec<Scope>,

    // template data
    layout: Option<LayoutTempl>,
    root: Vec<Stmt>,
    statics: Vec<String>,
}

impl<'a> Parser<'a> {
    /// create new [`Parser`]
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            index: 0,
            state: ParseState::Static { start: 0 },
            scopes: vec![],
            layout: None,
            root: vec![],
            statics: vec![],
        }
    }

    fn push_stack(&mut self, stmt: Stmt) {
        match self.scopes.last_mut() {
            Some(ok) => ok.stack(),
            None => &mut self.root,
        }.push(stmt);
    }

    /// start parsing
    pub fn parse(mut self) -> Result<Template> {
        loop {
            let current = self.index;
            let Some(byte) = self.source.get(current) else {
                break self.parse_leftover()?
            };

            match self.state {
                ParseState::Static { start } => {
                    self.index += 1;
                    if &b'{' == byte {
                        self.state = ParseState::OpenExpr { start, brace: current }
                    }
                },
                ParseState::Expr { start } => {
                    self.index += 1;
                    if &b'}' == byte {
                        self.state = ParseState::CloseExpr { start, brace: current }
                    }
                }
                ParseState::OpenExpr { start, brace, } => {
                    match byte {
                        b'{' | b'%' => {
                            self.index += 1;
                            self.state = ParseState::Expr { start: current + 1 };
                            self.collect_static(&self.source[start..brace])?;
                        },
                        _ => self.state = ParseState::Static { start },
                    }
                }
                ParseState::CloseExpr { start, brace } => {
                    match byte {
                        b'}' | b'%' => {
                            self.index += 1;
                            self.state = ParseState::Static { start: current + 1 };
                            self.parse_expr(&self.source[start..brace])?;
                        }
                        _ => self.state = ParseState::Expr { start },
                    }
                }
            }
        }

        if let Some(scope) = self.scopes.pop() {
            error!("unclosed `{scope}` scope")
        }

        Ok(Template {
            layout: self.layout,
            stmts: self.root,
            statics: self.statics,
        })
    }

    fn collect_static(&mut self, source: &[u8]) -> Result<()> {
        if source.is_empty() {
            return Ok(())
        }

        let source = parse_str(source).to_owned();
        let idx = Index::from(self.statics.len());
        let src = if cfg!(debug_assertions) {
            quote! {&sources[#idx]}
        } else {
            quote! {#source}
        };

        self.statics.push(source);
        self.push_stack(syn::parse_quote! { #Display(#src, writer)?; });

        Ok(())
    }

    /// track multiple stacked tokens
    /// add token stack when encounter starting scope
    /// and pop tokens when scope closes
    fn parse_expr(&mut self, source: &[u8]) -> Result<()> {
        match syn::parse_str(parse_str(source)).map_err(Error::Syn)? {
            ExprTempl::Layout(layout) => {
                if self.layout.is_some() {
                    error!("attempt to add 2 `layout`")
                }
                self.layout.replace(layout);
            }
            ExprTempl::Yield(_yield) => {
                self.push_stack(syn::parse_quote! {
                    #Display(&layout_inner, &mut *writer)?;
                });
            }
            ExprTempl::Expr(expr) => {
                self.push_stack(syn::parse_quote! {
                    #Display(&#expr, &mut ::tour::render::Escape(&mut *writer))?;
                });
            }
            ExprTempl::Unsafe(UnsafeTempl { unsafe_token: _, expr }) => {
                self.push_stack(syn::parse_quote! {
                    #Display(&#expr, writer)?;
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
                            Scope::Root { .. } => error!("attempt to add 2 `else`"),
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
                            error!("attempt to add 2 `else` in `for` scope")
                        }
                        else_branch.replace((else_token, Scope::Root { stmts: vec![] }.into()));
                        self.scopes.push(Scope::For { templ, stmts, else_branch });
                    }
                    Some(scope) => error!("attempt to close `else` in `{scope}` scope"),
                    None => error!("attempt to close `else` in toplevel"),
                };
            }

            ExprTempl::EndIf(_endif) => {
                let (IfTempl { if_token, cond },stmts,else_branch) = match self.scopes.pop() {
                    Some(Scope::If { templ, stmts, else_branch, }) => (templ,stmts,else_branch),
                    Some(scope) => error!("attempt to close `endif` in `{scope}` scope"),
                    None => error!("attempt to close `endif` in toplevel"),
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
                    Some(scope) => error!("attempt to close `else` in `{scope}` scope"),
                    None => error!("attempt to close `else` in toplevel"),
                };

                self.push_stack(syn::parse_quote!(let __for_expr = #expr;));

                if let Some((_,body)) = else_branch.map(else_branch_expr) {
                    self.push_stack(syn::parse_quote!(
                        if ExactSizeIterator::len(&IntoIterator::into_iter(&__for_expr)) == 0 #body
                    ));
                }

                self.push_stack(syn::parse_quote!(#for_token #pat #in_token __for_expr { #(#stmts)* }));
            }
        }
        Ok(())
    }

    fn parse_leftover(&mut self) -> Result<()> {
        match self.state {
            ParseState::Static { start } | ParseState::OpenExpr { start, .. } => {
                self.collect_static(&self.source[start..])
            }
            ParseState::Expr { start } | ParseState::CloseExpr { start, .. } => {
                self.parse_expr(&self.source[start..])
            }
        }
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

fn parse_str(v: &[u8]) -> &str {
    // SAFETY: parser input is a string, and we always
    // check byte by char, so str boundary is ok
    unsafe { core::str::from_utf8_unchecked(v) }
}

struct Display;

impl quote::ToTokens for Display {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote! {::tour::Display::display}.to_tokens(tokens);
    }
}

/// parsing result alias
pub type Result<T,E = Error> = std::result::Result<T,E>;

/// parsing error
#[derive(Debug)]
pub enum Error {
    Generic(String),
    Syn(syn::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(s) => f.write_str(s),
            Error::Syn(error) => error.fmt(f),
        }
    }
}

