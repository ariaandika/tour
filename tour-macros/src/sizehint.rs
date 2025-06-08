use syn::*;

use crate::{
    data::Template,
    syntax::{RenderTempl, RenderValue},
    visitor::{Scalar, Scope, StmtTempl},
};

pub type SizeHint = (usize,Option<usize>);

pub fn size_hint(templ: &Template) -> Result<(usize, Option<usize>)> {
    Visitor { templ }.visit_stmts(templ.stmts()?)
}

pub fn size_hint_block(templ: &Template, block: &Ident) -> Result<(usize, Option<usize>)> {
    Visitor { templ }.visit_stmts(&templ.try_block(block)?.stmts)
}

struct Visitor<'a> {
    templ: &'a Template,
}

impl Visitor<'_> {
    fn visit_stmts(&self, stmts: &[StmtTempl]) -> Result<(usize,Option<usize>)> {
        let mut size_hint = (0,None);
        for stmt in stmts {
            let size = self.visit_stmt(stmt)?;
            size_hint = add_size_hint(size_hint, size);
        }
        Ok(size_hint)
    }

    fn visit_stmt(&self, stmt: &StmtTempl) -> Result<(usize, Option<usize>)> {
        let size = match stmt {
            StmtTempl::Scalar(scalar) => match scalar {
                Scalar::Static(source, _) => (source.len(), Some(source.len())),
                Scalar::Render(RenderTempl { value, .. }) => match value {
                    RenderValue::Path(path) => {
                        let target = match path.get_ident() {
                            Some(id) => match self.templ.get_block(id) {
                                Some(block) => &block.stmts,
                                None => self.templ.get_import_by_alias(path)?.templ().stmts()?,
                            },
                            None => {
                                self.templ.get_import_by_alias(path)?.templ().stmts()?
                            },
                        };
                        self.visit_stmts(target)?
                    },
                    RenderValue::LitStr(_lit_str) => {
                        // TODO: calculate imported template size_hint
                        (0, None)
                    }
                },
                Scalar::Yield | Scalar::Expr(_, _) | Scalar::Use(_) | Scalar::Const(_) => (0, None),
            },
            StmtTempl::Scope(scope) => self.visit_scope(scope)?,
        };

        Ok(size)
    }

    fn visit_scope(&self, scope: &Scope) -> Result<(usize, Option<usize>)> {
        let size = match scope {
            Scope::Root { stmts } => self.visit_stmts(stmts)?,
            Scope::If { stmts, else_branch, .. } => {
                let s1 = self.visit_stmts(stmts)?;

                let s2 = if let Some((_, else_scope)) = else_branch {
                    self.visit_scope(else_scope)?
                } else {
                    (0, None)
                };

                merge_size_hint(s1, s2)
            }
            Scope::For { stmts, else_branch, .. } => {
                // iteration size hint calculation is:
                // either not iterated (else branch) or iterated once (main branch)
                //
                // more complex would be using `Iterator::size_hint`

                let main_size = self.visit_stmts(stmts)?;

                let else_size = match else_branch {
                    Some((_, else_scope)) => self.visit_scope(else_scope)?,
                    _ => (0, None),
                };

                merge_size_hint(main_size, else_size)
            },
            Scope::Block { .. } => unreachable!("`block` scope should be replaced with `render`"),
        };

        Ok(size)
    }
}

pub fn add_size_hint(s1: SizeHint, s2: SizeHint) -> SizeHint {
    (
        s1.0 + s2.0,
        match (s1.1,s2.1) {
            (None, None) => None,
            (None, max @ Some(_)) => max,
            (max @ Some(_), None) => max,
            (Some(max), Some(mx)) => Some(max + mx)
        }
    )
}

pub fn merge_size_hint(s1: SizeHint, s2: SizeHint) -> SizeHint {
    (
        s1.0.min(s2.0),
        match (s1.1,s2.1) {
            (None, None) => None,
            (None, max @ Some(_)) => max,
            (max @ Some(_), None) => max,
            (Some(max), Some(mx)) => Some(max.max(mx))
        }
    )
}

