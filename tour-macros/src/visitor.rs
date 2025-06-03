//! [`Visitor`] implementation via syn
use std::collections::HashMap;
use syn::*;
use tour_core::{Delimiter, ParseError, Result, Visitor};

use crate::{
    shared::SourceTempl,
    syntax::{ExprTempl, *},
};

macro_rules! error {
    ($($tt:tt)*) => {
        return Err(ParseError::Generic(format!($($tt)*)))
    };
}

// NOTE: this module should not do any codegen
// only collect all tokens in Visitor implementation

/// Contains a single file template information.
pub struct Template {
    pub layout: Option<SourceTempl>,
    pub blocks: HashMap<Ident, BlockContent>,
    pub statics: Vec<String>,
    pub stmts: Vec<StmtTempl>,
}

pub struct BlockContent {
    #[allow(unused)]
    pub templ: BlockTempl,
    pub stmts: Vec<StmtTempl>,
}

pub enum StmtTempl {
    Scalar(Scalar),
    Scope(Scope),
}

pub enum Scalar {
    Static(String,Index),
    Expr(Expr,Delimiter),
    Render(RenderTempl),
    Use(UseTempl),
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

pub struct SynVisitor {
    layout: Option<SourceTempl>,
    blocks: HashMap<Ident, BlockContent>,
    statics: Vec<String>,
    root: Vec<StmtTempl>,

    /// currently open scopes
    scopes: Vec<Scope>,
}

impl SynVisitor {
    pub fn new() -> Self {
        Self {
            layout: None,
            blocks: <_>::default(),
            statics: vec![],
            root: vec![],
            scopes: vec![],
        }
    }

    pub fn finish(self) -> Template {
        Template {
            layout: self.layout,
            blocks: self.blocks,
            statics: self.statics,
            stmts: self.root,
        }
    }

    fn stack_mut(&mut self) -> &mut Vec<StmtTempl> {
        match self.scopes.last_mut() {
            Some(ok) => ok.stack(),
            None => &mut self.root,
        }
    }
}

impl Visitor<'_> for SynVisitor {
    fn visit_static(&mut self, source: &str) -> Result<()> {
        let idx = Index::from(self.statics.len());
        self.stack_mut().push(StmtTempl::Scalar(Scalar::Static(source.to_owned(),idx)));
        self.statics.push(source.to_owned());
        Ok(())
    }

    fn visit_expr(&mut self, source: &str, delim: Delimiter) -> Result<()> {
        let expr = match syn::parse_str(source) {
            Ok(ok) => ok,
            Err(err) => error!("failed to parse expr: {err}"),
        };

        match expr {
            // ===== layout =====

            ExprTempl::Layout(LayoutTempl { root_token, source, .. }) |
            ExprTempl::Extends(ExtendsTempl { root_token, source, .. }) => {
                let dupl = self.layout.replace(if root_token.is_some() {
                    SourceTempl::Root(source.value())
                } else {
                    SourceTempl::Path(source.value())
                });

                if dupl.is_some() {
                    error!("cannot have 2 `extends` or `layout`")
                }
            },

            // ==== scalar =====

            ExprTempl::Yield(_yield) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Yield));
            },
            ExprTempl::Render(templ) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Render(templ)));
            },
            ExprTempl::Expr(expr) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Expr(expr,delim)));
            },
            ExprTempl::Use(templ) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Use(templ)));
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

                self.blocks.insert(name.clone(), BlockContent { templ, stmts });
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Render(
                    RenderTempl {
                        render_token: <_>::default(),
                        name,
                    },
                )));
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

