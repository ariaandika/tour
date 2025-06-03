use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::HashMap;
use syn::*;

use crate::{
    attribute::AttrData,
    shared::{self, error, TemplDisplay},
    syntax::RenderTempl,
    visitor::{BlockContent, Scalar, Scope, StmtTempl, Template},
};

pub fn generate(attr: &AttrData, templ: &Template) -> Result<TokenStream> {
    // blocks cannot be in `Visitor`, because its possible to `visit` statements inside
    // `blocks` which take mutable reference of the whole `Visitor`.
    let shared = Shared {
        attr,
        blocks: &templ.blocks,
    };

    let mut visitor = Visitor::default();
    visitor.visit_stmts(&templ.stmts, &shared)?;

    Ok(visitor.tokens)
}

struct Shared<'a> {
    attr: &'a AttrData,
    blocks: &'a HashMap<Ident, BlockContent>,
}

#[derive(Default)]
struct Visitor {
    tokens: TokenStream,
    static_len: usize,
}

impl Visitor {
    fn with_statics_len(static_len: usize) -> Self {
        Self { tokens: <_>::default(), static_len }
    }

    fn visit_stmts(&mut self, stmts: &[StmtTempl], shared: &Shared) -> Result<()> {
        for stmt in stmts {
            self.visit_stmt(stmt, shared)?;
        }
        Ok(())
    }

    fn visit_stmt(&mut self, stmt: &StmtTempl, shared: &Shared) -> Result<()> {
        match stmt {
            StmtTempl::Scalar(scalar) => match scalar {
                Scalar::Static(source) => {
                    let idx = Index::from(self.static_len);
                    let src = match shared.attr.reload.as_bool() {
                        Ok(cond) => if cond { quote! {&sources[#idx]} } else { quote! {#source} },
                        Err(expr) => quote! { if #expr { &sources[#idx] } else { #source } },
                    };

                    self.static_len += 1;
                    self.tokens.extend(quote! {
                        #TemplDisplay::display(#src, writer)?;
                    });
                },
                Scalar::Yield => {
                    self.tokens.extend(quote! {
                        #TemplDisplay::display(&self.0, &mut *writer)?;
                    });
                },
                Scalar::Render(RenderTempl { name, .. }) => {
                    let Some(block) = shared.blocks.get(name) else {
                        error!("block `{name}` not found")
                    };
                    self.visit_stmts(&block.stmts, shared)?;
                },
                Scalar::Expr(expr, delim) => {
                    let display = shared::display_ref(*delim, expr);
                    let writer = shared::writer(*delim);
                    self.tokens.extend(quote! {
                        #TemplDisplay::display(#display, #writer)?;
                    });
                },
                Scalar::Use(templ) => {
                    templ.use_token.to_tokens(&mut self.tokens);
                    templ.path.to_tokens(&mut self.tokens);
                    quote::__private::push_semi(&mut self.tokens);
                },
            },
            StmtTempl::Scope(scope) => self.visit_scope(scope, shared)?,
        }

        Ok(())
    }

    fn visit_scope(&mut self, scope: &Scope, shared: &Shared) -> Result<()> {
        match scope {
            Scope::Root { stmts } => {
                let mut visitor = Visitor::with_statics_len(self.static_len);
                visitor.visit_stmts(stmts, shared)?;

                self.static_len = visitor.static_len;
                token::Brace::default()
                    .surround(&mut self.tokens, |t|*t=visitor.tokens);
            },
            Scope::If { templ, stmts, else_branch } => {
                templ.if_token.to_tokens(&mut self.tokens);
                templ.cond.to_tokens(&mut self.tokens);

                let mut visitor = Visitor::with_statics_len(self.static_len);
                visitor.visit_stmts(stmts, shared)?;

                self.static_len = visitor.static_len;
                token::Brace::default()
                    .surround(&mut self.tokens, |t|*t=visitor.tokens);

                if let Some((else_token, else_scope)) = else_branch {
                    else_token.to_tokens(&mut self.tokens);
                    self.visit_scope(else_scope, shared)?;
                }
            },
            Scope::For { templ, stmts, else_branch } => {
                let expr = &templ.expr;

                self.tokens.extend(quote! {
                    let __for_expr = #expr;
                });

                templ.for_token.to_tokens(&mut self.tokens);
                templ.pat.to_tokens(&mut self.tokens);
                templ.in_token.to_tokens(&mut self.tokens);
                format_ident!("__for_expr").to_tokens(&mut self.tokens);

                let mut visitor = Visitor::with_statics_len(self.static_len);
                visitor.visit_stmts(stmts, shared)?;

                self.static_len = visitor.static_len;
                token::Brace::default()
                    .surround(&mut self.tokens, |t|*t=visitor.tokens);

                if let Some((_, else_scope)) = else_branch {
                    self.tokens.extend(quote! {
                        if ExactSizeIterator::len(&IntoIterator::into_iter(__for_expr)) == 0
                    });

                    self.visit_scope(else_scope, shared)?;
                }
            },
            Scope::Block { .. } => unreachable!("`block` scope should be replaced with `render`")
        }

        Ok(())
    }
}

