use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Token;

use super::paren;
use crate::{
    ast::{Scalar, Scope, StmtTempl},
    data::Template,
    file::AliasKind,
    syntax::{RenderTempl, RenderValue},
};

pub fn generate(size: SizeHint, tokens: &mut TokenStream) {
    paren(tokens, |tokens| {
        let (min,max) = size;
        min.to_tokens(tokens);
        <Token![,]>::default().to_tokens(tokens);
        match max {
            Some(max) => tokens.extend(quote! { Some(#max) }),
            None => tokens.extend(quote! { None }),
        }
    });
}

pub fn is_empty(size: SizeHint) -> bool {
    matches!(size,(0,None))
}

pub type SizeHint = (usize, Option<usize>);

pub struct Visitor<'a> {
    templ: &'a Template,
}

impl<'a> Visitor<'a> {
    pub fn new(templ: &'a Template) -> Self {
        Self { templ }
    }
 
    pub fn calculate(&self) -> SizeHint {
        self.visit_stmts(self.templ.stmts())
    }

    pub fn calculate_block(&self, block: &syn::Ident) -> SizeHint {
        self.visit_stmts(&self.templ.file().block(block).stmts)
    }

    fn visit_stmts(&self, stmts: &[StmtTempl]) -> SizeHint {
        let mut size = (0,None);
        for stmt in stmts {
            size = add(size, self.visit_stmt(stmt));
        }
        size
    }

    fn visit_stmt(&self, stmt: &StmtTempl) -> SizeHint {
        match stmt {
            StmtTempl::Scalar(scalar) => match scalar {
                Scalar::Static { value, .. } => exact(value.len()),
                Scalar::Render(RenderTempl { value: RenderValue::Ident(id), block, .. }) => {
                    match (self.templ.file().resolve_id(id), block) {
                        (AliasKind::Block(block), None) => self.visit_stmts(&block.stmts),
                        (AliasKind::Block(_), Some(_)) => unreachable!("cannot render block from block"),
                        (AliasKind::Import(import), None) => {
                            let me = Visitor { templ: import.templ() };
                            me.visit_stmts(import.templ().stmts())
                        },
                        (AliasKind::Import(import), Some((_, block))) => {
                            let block = import.templ().file().block(block);
                            let me = Visitor { templ: import.templ() };
                            me.visit_stmts(&block.stmts)
                        },
                    }
                },
                Scalar::Render(RenderTempl { value: RenderValue::Path(path), block, .. }) => {
                    let templ = self.templ.file().import_by_path(path).templ();
                    match block {
                        Some((_, block)) => {
                            let block = templ.file().block(block);
                            self.visit_stmts(&block.stmts)
                        },
                        None => {
                            self.visit_stmts(templ.stmts())
                        },
                    }
                },
                Scalar::Yield(_) | Scalar::Expr { .. } | Scalar::Use(_) | Scalar::Item(_) => (0,None),
            },
            StmtTempl::Scope(scope) => self.visit_scope(scope),
        }
    }

    fn visit_scope(&self, scope: &Scope) -> SizeHint {
        match scope {
            Scope::Root { stmts } => self.visit_stmts(stmts),
            Scope::If { stmts, else_branch, .. } => {

                let s1 = self.visit_stmts(stmts);

                let s2 = if let Some((_, else_scope)) = else_branch {
                    self.visit_scope(else_scope)
                } else {
                    (0, None)
                };

                merge(s1, s2)
            }
            Scope::For { stmts, else_branch, .. } => {
                // iteration size hint calculation is:
                // either not iterated (else branch) or iterated once (main branch)
                //
                // more complex would be using `Iterator::size_hint`

                let main_size = self.visit_stmts(stmts);

                let else_size = match else_branch {
                    Some((_, else_scope)) => self.visit_scope(else_scope),
                    _ => (0, None),
                };

                merge(main_size, else_size)
            },
            Scope::Block { .. } => unreachable!("`block` scope should be replaced with `render`"),
        }
    }
}

pub fn exact(len: usize) -> SizeHint {
    (len,Some(len))
}

pub fn add(s1: SizeHint, s2: SizeHint) -> SizeHint {
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

pub fn merge(s1: SizeHint, s2: SizeHint) -> SizeHint {
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

