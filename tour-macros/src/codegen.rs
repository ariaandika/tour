use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::*;
use tour_core::Delimiter;

use crate::{
    common::{TemplDisplay, TemplWrite},
    data::{AliasKind, Template},
    syntax::{RenderTempl, RenderValue, UseValue},
    visitor::{Scalar, Scope, StmtTempl},
};

pub fn generate(templ: &Template) -> Result<TokenStream> {
    let shared = Shared {
        templ,
    };

    let mut visitor = Visitor::default();
    visitor.visit_stmts(templ.stmts()?, &shared)?;

    Ok(visitor.tokens)
}

pub fn generate_block(templ: &Template, block: &Ident) -> Result<TokenStream> {
    let shared = Shared {
        templ,
    };

    let mut visitor = Visitor::default();
    visitor.visit_stmts(&templ.try_block(block)?.stmts, &shared)?;

    Ok(visitor.tokens)
}

/// Generate new wrapper type for external template.
///
/// `inner` should declare their own lifetime requirements.
///
/// Lifetime `'a` is available.
///
/// `Body<'a>` or `&'a Body`
pub fn generate_typed_template(name: impl ToTokens, inner: impl ToTokens, body: impl ToTokens) -> TokenStream {
    quote! {
        struct #name<'a>(#inner);

        #[automatically_derived]
        impl<'a> std::ops::Deref for #name<'a> {
            type Target = #inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[automatically_derived]
        impl #TemplDisplay for #name<'_> {
            fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                #body
            }
        }
    }
}

struct Shared<'a> {
    templ: &'a Template,
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
                Scalar::Static(source, idx) => {
                    let idx = Index::from(*idx as usize);
                    let src = match shared.templ.reload().as_bool() {
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
                Scalar::Render(RenderTempl { value, .. }) => match value {
                    RenderValue::Path(path) => match shared.templ.resolve_alias(path)? {
                        AliasKind::Block(block) => self.visit_stmts(&block.stmts, shared)?,
                        AliasKind::Import(import) => {
                            let name = import.generate_name();
                            self.tokens.extend(quote! {
                                #TemplDisplay::display(&#name(self), &mut *writer)?;
                            });
                        }
                    },
                    RenderValue::LitStr(lit_str) => {
                        let import = shared.templ.try_import_by_path(lit_str)?;
                        let name = import.generate_name();
                        self.tokens.extend(quote! {
                            #TemplDisplay::display(&#name(self), &mut *writer)?;
                        });
                    },
                },
                Scalar::Expr(expr, delim) => {
                    let display = display(*delim, expr);
                    let writer = writer(*delim);
                    self.tokens.extend(quote! {
                        #TemplDisplay::display(#display, #writer)?;
                    });
                },
                Scalar::Use(templ) => match &templ.value {
                    UseValue::Tree(leading_colon, tree) => {
                        templ.use_token.to_tokens(&mut self.tokens);
                        leading_colon.to_tokens(&mut self.tokens);
                        tree.to_tokens(&mut self.tokens);
                        templ.semi_token.unwrap_or_default().to_tokens(&mut self.tokens);
                    },
                    UseValue::Alias(_) => unreachable!("use alias statement should be discarded")
                },
                Scalar::Const(templ) => {
                    templ.const_token.to_tokens(&mut self.tokens);
                    templ.ident.to_tokens(&mut self.tokens);
                    templ.colon_token.to_tokens(&mut self.tokens);
                    templ.ty.to_tokens(&mut self.tokens);
                    templ.eq.to_tokens(&mut self.tokens);
                    templ.expr.to_tokens(&mut self.tokens);
                    templ.semi_token.unwrap_or_default().to_tokens(&mut self.tokens);
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

fn display(delim: Delimiter, expr: &syn::Expr) -> TokenStream {
    use Delimiter::*;

    match delim {
        Quest => quote! {&::tour::Debug(&#expr)},
        Percent => quote! {&::tour::Display(&#expr)},
        Brace | Bang | Hash => quote! {&#expr},
    }
}

fn writer(delim: Delimiter) -> TokenStream {
    use Delimiter::*;

    match delim {
        Bang => quote! {&mut *writer},
        Brace | Percent | Quest | Hash => quote! {&mut ::tour::Escape(&mut *writer)},
    }
}
