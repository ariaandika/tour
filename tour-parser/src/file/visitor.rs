//! [`Visitor`] implementation via syn
use std::rc::Rc;
use syn::*;
use tour_core::{Delimiter, ParseError, Parser, Result, Visitor};

use super::{BlockContent, File, Import};
use crate::{
    ast::{Scalar, Scope, StmtTempl},
    data::Template,
    metadata::Metadata,
    syntax::*,
};

macro_rules! error {
    ($($tt:tt)*) => {
        return Err(ParseError::Generic(format!($($tt)*)))
    };
}

// ===== Visitor =====

pub struct SynVisitor<'a> {
    layout: Option<LayoutTempl>,
    imports: Vec<Import>,
    blocks: Vec<BlockContent>,
    statics: Vec<Rc<str>>,
    root: Vec<StmtTempl>,

    /// currently open scopes
    scopes: Vec<Scope>,
    meta: &'a Metadata,
}

impl<'a> SynVisitor<'a> {
    pub fn generate(meta: &Metadata) -> syn::Result<File> {
        let source = meta.resolve_source()?;
        let visitor = SynVisitor {
            layout: None,
            imports: vec![],
            blocks: vec![],
            statics: vec![],
            root: vec![],
            scopes: vec![],
            meta,
        };
        let ok = crate::common::error!(!Parser::new(source.as_ref(), visitor).parse());
        let SynVisitor { layout, imports, blocks, statics, root, .. } = ok;
        Ok(File { layout, imports, blocks, statics, stmts: root })
    }

    fn stack_mut(&mut self) -> &mut Vec<StmtTempl> {
        match self.scopes.last_mut() {
            Some(ok) => ok.stack_mut(),
            None => &mut self.root,
        }
    }

    fn import(&mut self, lit_str: &LitStr) -> Result<()> {
        self.import_only(lit_str, crate::common::name())
    }

    fn import_aliased(&mut self, alias: &UseTempl) -> Result<()> {
        self.import_only(&alias.path, alias.ident.clone())
    }

    fn import_only(&mut self, path: &LitStr, alias: Ident) -> Result<()> {
        let path: Rc<str> = path.value().into();

        if !self.imports.iter().any(|e|e==&*path) {
            let meta = self.meta.clone_with_path(&*path);
            let file = match Self::generate(&meta) {
                Ok(ok) => ok,
                Err(err) => return Err(ParseError::Generic(err.to_string())),
            };
            let templ = match Template::new(alias.clone(), meta, file) {
                Ok(ok) => ok,
                Err(err) => error!("{err}"),
            };
            self.imports.push(Import::new(path, alias, templ));
        }

        Ok(())
    }
}

impl Visitor<'_> for SynVisitor<'_> {
    fn visit_static(&mut self, source: &str) -> Result<()> {
        let index = self.statics.len().try_into().unwrap();

        self.stack_mut().push(StmtTempl::Scalar(Scalar::Static {
            value: source.into(),
            index,
        }));
        self.statics.push(source.into());

        Ok(())
    }

    fn visit_expr(&mut self, source: &str, delim: Delimiter) -> Result<()> {
        let expr = match syn::parse_str(source) {
            Ok(ok) => ok,
            Err(err) => error!("failed to parse expr: {err}"),
        };

        match expr {
            // ===== external reference =====

            StmtSyn::Layout(new_layout) => {
                let path = new_layout.path.clone();
                if self.layout.replace(new_layout).is_some() {
                    error!("cannot have 2 `extends` or `layout`")
                }
                self.import(&path)?;
            },
            StmtSyn::Use(templ) => self.import_aliased(&templ)?,
            StmtSyn::Render(templ) => {
                if let RenderValue::Path(lit_str) = &templ.value {
                    self.import(lit_str)?;
                }
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Render(templ)));
            },

            // ===== scalar =====

            StmtSyn::Yield(_yield) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Yield));
            },
            StmtSyn::Item(item) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Item(item)));
            },
            StmtSyn::Expr(expr) => {
                self.stack_mut().push(StmtTempl::Scalar(Scalar::Expr { expr, delim, }));
            },

            // ===== open scope =====

            StmtSyn::Block(templ) => {
                self.scopes.push(Scope::Block { templ, stmts: vec![] });
            },
            StmtSyn::If(templ) => {
                self.scopes.push(Scope::If { templ, stmts: vec![], else_branch: None, });
            },
            StmtSyn::For(templ) => {
                self.scopes.push(Scope::For { templ, stmts: vec![], else_branch: None, });
            },

            // ===== else / intermediate scope =====

            StmtSyn::Else(ElseTempl { else_token, elif_branch }) => {
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

            StmtSyn::Endblock(_endblock) => {
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
                            value: RenderValue::Ident(name),
                            block: None,
                        },
                    )));
                }

                self.blocks.push(BlockContent { templ, stmts });
            },
            StmtSyn::EndIf(_endif) => {
                let if_scope = match self.scopes.pop() {
                    Some(templ @ Scope::If { .. }) => templ,
                    Some(scope) => error!("cannot close `endif` in `{scope}` scope"),
                    None => error!("cannot close `endif` in toplevel"),
                };

                self.stack_mut().push(StmtTempl::Scope(if_scope));
            },
            StmtSyn::EndFor(_endfor) => {
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
