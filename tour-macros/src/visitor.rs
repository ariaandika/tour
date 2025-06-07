//! [`Visitor`] implementation via syn
use syn::*;
use tour_core::{Delimiter, ParseError, Parser, Result, Visitor};

use crate::{data::{BlockContent, File, Metadata, Template}, syntax::*};

macro_rules! error {
    ($($tt:tt)*) => {
        return Err(ParseError::Generic(format!($($tt)*)))
    };
}

pub fn generate_file(meta: &Metadata) -> syn::Result<File> {
    let source = meta.resolve_source()?;
    let ok = crate::common::error!(!Parser::new(source.as_ref(), SynVisitor::new(meta)).parse());

    let SynVisitor { layout, imports, blocks, statics, root, .. } = ok;
    Ok(File::new(layout, imports, blocks, statics, root))
}

// ===== ImportKey =====

pub struct Import {
    path: Box<str>,
    alias: Option<Ident>,
    pub templ: Template,
}

impl PartialEq<str> for Import {
    fn eq(&self, other: &str) -> bool {
        self.path.as_ref() == other
    }
}

impl PartialEq<Path> for Import {
    fn eq(&self, other: &Path) -> bool {
        let other = other.segments.first().map(|e|&e.ident);
        match (self.alias.as_ref(), other) {
            (Some(me), Some(other)) => me == other,
            _ => false,
        }
    }
}

// ===== Nested Syntax =====

pub enum StmtTempl {
    Scalar(Scalar),
    Scope(Scope),
}

pub enum Scalar {
    Static(Box<str>,u32),
    Expr(Box<Expr>,Delimiter),
    Render(RenderTempl),
    Use(UseTempl),
    Const(ConstTempl),
    Yield,
}

pub enum Scope {
    Root { stmts: Vec<StmtTempl> },
    If {
        templ: IfTempl,
        stmts: Vec<StmtTempl>,
        else_branch: Option<(Token![else],Box<Scope>)>
    },
    For {
        templ: ForTempl,
        stmts: Vec<StmtTempl>,
        else_branch: Option<(Token![else],Box<Scope>)>
    },
    Block {
        templ: BlockTempl,
        stmts: Vec<StmtTempl>,
    },
}

impl Scope {
    fn stack(&mut self) -> &mut Vec<StmtTempl> {
        match self {
            Self::Root { stmts } => stmts,
            Self::Block { stmts, .. } => stmts,
            Self::For { else_branch: Some(branch), .. } => branch.1.stack(),
            Self::For { stmts, .. } => stmts,
            Self::If { else_branch: Some(branch), .. } => branch.1.stack(),
            Self::If { stmts, .. } => stmts,
        }
    }
}

// ===== Visitor =====

pub struct SynVisitor<'a> {
    layout: Option<LayoutTempl>,
    imports: Vec<Import>,
    blocks: Vec<BlockContent>,
    statics: Vec<Box<str>>,
    root: Vec<StmtTempl>,

    /// currently open scopes
    scopes: Vec<Scope>,
    meta: &'a Metadata,
}

impl<'a> SynVisitor<'a> {
    pub fn new(meta: &'a Metadata) -> Self {
        Self {
            layout: None,
            imports: <_>::default(),
            blocks: vec![],
            statics: vec![],
            root: vec![],
            scopes: vec![],
            meta,
        }
    }

    fn stack_mut(&mut self) -> &mut Vec<StmtTempl> {
        match self.scopes.last_mut() {
            Some(ok) => ok.stack(),
            None => &mut self.root,
        }
    }

    fn import(&mut self, lit_str: &LitStr) -> Result<()> {
        let path = lit_str.value().into_boxed_str();
        if !self.imports.iter().any(|e|e==&*path) {
            let meta = self.meta.clone_with_path(&*path);
            let file = match generate_file(&meta) {
                Ok(ok) => ok,
                Err(err) => return Err(ParseError::Generic(err.to_string())),
            };
            self.imports.push(Import {
                path,
                alias: None,
                templ: Template::new(meta, file),
            });
        }
        Ok(())
    }
}

impl Visitor<'_> for SynVisitor<'_> {
    fn visit_static(&mut self, source: &str) -> Result<()> {
        let idx = self.statics.len().try_into().unwrap();
        self.stack_mut().push(StmtTempl::Scalar(Scalar::Static(source.into(), idx)));
        self.statics.push(source.into());
        Ok(())
    }

    fn visit_expr(&mut self, source: &str, delim: Delimiter) -> Result<()> {
        let expr = match syn::parse_str(source) {
            Ok(ok) => ok,
            Err(err) => error!("failed to parse expr: {err}"),
        };

        match expr {
            // ===== layout =====

            ExprTempl::Layout(layout) => {
                if self.layout.replace(layout).is_some() {
                    error!("cannot have 2 `extends` or `layout`")
                }
            },

            // ===== external reference =====

            ExprTempl::Render(templ) => {
                if let RenderValue::LitStr(lit_str) = &templ.value {
                    self.import(lit_str)?;
                }
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Render(templ)));
            },
            ExprTempl::Use(templ) => {
                match templ.value {
                    UseValue::Tree(_, _) => self.stack_mut().push(StmtTempl::Scalar(Scalar::Use(templ))),
                    UseValue::LitStr(lit_str) => self.import(&lit_str)?,
                }
            },

            // ===== scalar =====

            ExprTempl::Yield(_yield) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Yield));
            },
            ExprTempl::Expr(expr) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Expr(expr,delim)));
            },
            ExprTempl::Const(templ) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Const(templ)));
            },

            // ===== open scope =====

            ExprTempl::Block(templ) => {
                self.scopes.push(Scope::Block { templ, stmts: vec![] });
            },
            ExprTempl::If(templ) => {
                self.scopes.push(Scope::If { templ, stmts: vec![], else_branch: None, });
            },
            ExprTempl::For(templ) => {
                self.scopes.push(Scope::For { templ, stmts: vec![], else_branch: None, });
            },

            // ===== else / intermediate scope =====

            ExprTempl::Else(ElseTempl { else_token, elif_branch }) => {
                type ElseBranch = Option<(Token![else], Box<Scope>)>;

                fn take_latest_else_branch(else_branch: &mut ElseBranch) -> Result<&mut ElseBranch> {
                    match else_branch {
                        Some((_, branch)) => match &mut **branch {
                            // previously `else if`, we can fill more else branches
                            Scope::If { else_branch, .. } => take_latest_else_branch(else_branch),
                            Scope::Root { .. } => error!("cannot have 2 `else`"),
                            _ => panic!("`else` scope can only contain Root or If"),
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
                        let dupl = else_branch.replace(
                            (else_token, Scope::Root { stmts: vec![] }.into())
                        );
                        if dupl.is_some() {
                            error!("cannot have 2 `else` in `for` scope")
                        }
                        self.scopes.push(Scope::For { templ, stmts, else_branch });
                    }
                    Some(scope) => error!("cannot close `else` in `{scope}` scope"),
                    None => error!("cannot close `else` in toplevel"),
                };
            },

            // ===== close scope =====

            ExprTempl::Endblock(_endblock) => {
                let (templ,stmts) = match self.scopes.pop() {
                    Some(Scope::Block { templ, stmts }) => (templ,stmts),
                    Some(scope) => error!("cannot close `endblock` in `{scope}` scope"),
                    None => error!("cannot close `endblock` in toplevel"),
                };

                let name = templ.name.clone();

                if templ.static_token.is_none() {
                    self.stack_mut().push(StmtTempl::Scalar(Scalar::Render(
                        RenderTempl {
                            render_token: <_>::default(),
                            value: RenderValue::Path(name.into()),
                        },
                    )));
                }

                self.blocks.push(BlockContent { templ, stmts });
            },
            ExprTempl::EndIf(_endif) => {
                let if_scope = match self.scopes.pop() {
                    Some(templ @ Scope::If { .. }) => templ,
                    Some(scope) => error!("cannot close `endif` in `{scope}` scope"),
                    None => error!("cannot close `endif` in toplevel"),
                };

                self.stack_mut().push(StmtTempl::Scope(if_scope));
            },
            ExprTempl::EndFor(_endfor) => {
                let for_scope = match self.scopes.pop() {
                    Some(templ @ Scope::For { .. }) => templ,
                    Some(scope) => error!("cannot close `endfor` in `{scope}` scope"),
                    None => error!("cannot close `endfor` in toplevel"),
                };

                self.stack_mut().push(StmtTempl::Scope(for_scope));
            },
        }

        Ok(())
    }

    fn finish(mut self) -> Result<Self> {
        if let Some(scope) = self.scopes.pop() {
            error!("unclosed `{scope}` scope")
        }

        Ok(self)
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Root { .. } => f.write_str("root"),
            Self::Block { .. } => f.write_str("block"),
            Self::If { .. } => f.write_str("if"),
            Self::For { .. } => f.write_str("for"),
        }
    }
}

