use syn::{Ident, LitStr, Result};

use super::File;
use crate::{
    ast::{Scalar, Scope, StmtTempl},
    common::error,
    syntax::RenderValue,
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

    /// Try to render block or import
    fn validate_render_id(&self, id: &Ident) -> Result<()> {
        if self.file.get_block(id).is_some() {
            return Ok(())
        }

        if self.file.get_import_by_id(id).is_some() {
            return Ok(())
        }

        error!(id, "cannot find block/template `{id}`")
    }

    /// Try to render block or import
    fn validate_render_path(&self, path: &LitStr) -> Result<()> {
        if self.file.get_import_by_path(path).is_some() {
            return Ok(())
        }

        error!(path, "cannot find template `{}`",path.value())
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
                Scalar::Render(render) => match &render.value {
                    RenderValue::Ident(ident) => self.validate_render_id(ident)?,
                    RenderValue::Path(path) => self.validate_render_path(path)?,
                }
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



