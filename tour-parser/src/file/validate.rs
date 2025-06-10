use syn::Result;

use super::{AliasKind, File};
use crate::{
    ast::{Scalar, Scope, StmtTempl},
    common::error,
    syntax::{RenderTempl, RenderValue},
};

/// Template validation.
///
/// Validate early to allows for infallible codegen.
///
/// Will validate:
///
/// - the referenced template by `render` statement exists
pub struct ValidateVisitor<'a> {
    file: &'a File,
}

impl<'a> ValidateVisitor<'a> {
    /// Start validating.
    pub fn validate(file: &'a File) -> Result<()> {
        let me = Self { file };
        me.visit_stmts(&me.file.stmts)
    }

    fn visit_stmts(&self, stmts: &[StmtTempl]) -> Result<()> {
        for stmt in stmts {
            self.visit_stmt(stmt)?;
        }
        Ok(())
    }

    fn visit_stmt(&self, stmt: &StmtTempl) -> Result<()> {
        match stmt {
            StmtTempl::Scalar(scalar) => match scalar {
                Scalar::Static { .. } => {}
                Scalar::Use(_) => {}
                Scalar::Render(RenderTempl { value: RenderValue::Ident(id), block, .. }) => {
                    match (self.file.get_resolved_id(id),block) {
                        (Some(_), None) => {
                            // Ok
                        },
                        (Some(AliasKind::Import(import)), Some((_, block))) => {
                            if import.templ.file().get_block(block).is_none() {
                                error!(id, "cannot find block `{block}` in `{id}`")
                            }
                        },
                        (Some(AliasKind::Block(_)), Some((_, block))) => {
                            error!(id, "cannot render a block `{id}` from a block `{block}`")
                        },
                        (None,_) => {
                            error!(id, "cannot find block/template `{id}`")
                        }
                    }
                },
                Scalar::Render(RenderTempl { value: RenderValue::Path(path), block, .. }) => {
                    match (self.file.get_import_by_path(path), block) {
                        (Some(_), None) => {
                            // Ok
                        },
                        (Some(import), Some((_, block))) => {
                            if import.templ.file().get_block(block).is_none() {
                                error!(path, "cannot find block `{block}` in `{}`", path.value())
                            }
                        },
                        (None,_) => {
                            error!(path, "cannot find template `{}`", path.value())
                        },
                    }
                },
                Scalar::Yield => {}
                Scalar::Item(_) => {}
                Scalar::Expr { .. } => {}
            }
            StmtTempl::Scope(scope) => self.visit_scope(scope)?,
        }

        Ok(())
    }

    fn visit_scope(&self, scope: &Scope) -> Result<()> {
        match scope {
            Scope::Root { stmts } => self.visit_stmts(stmts)?,
            Scope::If { stmts, else_branch, .. } => {
                self.visit_stmts(stmts)?;
                if let Some((_,scope)) = else_branch {
                    self.visit_scope(scope)?;
                }
            },
            Scope::For { stmts, else_branch, .. } => {
                self.visit_stmts(stmts)?;
                if let Some((_,scope)) = else_branch {
                    self.visit_scope(scope)?;
                }
            },
            Scope::Block { .. } => unreachable!("`block` scope should be replaced with `render`")
        }

        Ok(())
    }
}



