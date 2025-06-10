use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::*;
use tour_core::Delimiter;

use crate::{
    ast::*,
    common::TemplDisplay,
    data::Template,
    file::AliasKind,
    syntax::{ItemTempl, RenderTempl, RenderValue},
};

use super::brace;

pub struct Visitor<'a> {
    tokens: &'a mut TokenStream,
    static_len: usize,
}

struct Shared<'a> {
    templ: &'a Template,
    input: &'a DeriveInput,
}

impl<'a> Visitor<'a> {
    pub fn generate(templ: &'a Template, input: &'a DeriveInput, tokens: &'a mut TokenStream) {
        let mut me = Self { tokens, static_len: 0, };
        let shared = Shared { templ, input };
        me.gens(templ.stmts(), &shared);
    }

    pub fn generate_block(templ: &'a Template, block: &Ident, input: &'a DeriveInput, tokens: &'a mut TokenStream) {
        let mut me = Self { tokens, static_len: 0, };
        let shared = Shared { templ, input };
        me.gens(&templ.file().block(block).stmts, &shared);
    }

    fn gens(&mut self, stmts: &[StmtTempl], shared: &Shared) {
        self.gen_destructure(shared);
        self.gen_sources(shared);
        self.visit_stmts(stmts, shared);
        self.tokens.extend(quote! {
            Ok(())
        });
    }

    fn gen_destructure(&mut self, shared: &Shared) {
        match &shared.input.data {
            Data::Struct(data) if matches!(data.fields, Fields::Named(_)) => {
                let ty = &shared.input.ident;
                self.tokens.extend(quote! { let #ty });

                brace(self.tokens, |tokens| {
                    for field in &data.fields {
                        field.ident.as_ref().expect("named").to_tokens(tokens);
                        <Token![,]>::default().to_tokens(tokens);
                    }
                });

                self.tokens.extend(quote! { = self; });
            }
            _ => {}
        }
    }

    fn gen_sources(&mut self, shared: &Shared) {
        let meta = shared.templ.meta();
        let path = meta.path();
        let statics = shared.templ.file().statics();
        match (meta.is_file(), meta.reload().as_bool()) {
            (true,Ok(true)) => self.tokens.extend(quote!{
                let sources = ::std::fs::read_to_string(#path)?;
                let sources = ::tour::Parser::new(&sources, ::tour::StaticVisitor::new())
                    .parse()?.statics;
            }),
            (true,Ok(false)) | (false,Ok(false)) => {}
            (true, Err(cond)) => self.tokens.extend(quote! {
                let sources = if #cond {
                    let sources = ::std::fs::read_to_string(#path)?;
                    ::tour::Parser::new(&sources, ::tour::StaticVisitor::new())
                        .parse()?.statics
                } else {
                    vec![]
                };
            }),
            (false, _) if statics.is_empty() => {}
            (false, _) => self.tokens.extend(quote! {
                let sources = [#(#statics),*];
            }),
        }
    }

    fn visit_stmts(&mut self, stmts: &[StmtTempl], shared: &Shared) {
        for stmt in stmts {
            self.visit_stmt(stmt, shared);
        }
    }

    fn visit_stmt(&mut self, stmt: &StmtTempl, shared: &Shared) {
        match stmt {
            StmtTempl::Scalar(scalar) => match scalar {
                Scalar::Static { value, index } => {
                    let idx = Index::from(*index as usize);

                    match shared.templ.meta().reload().as_bool() {
                        Ok(true) => self.tokens.extend(quote! {
                            #TemplDisplay::display(&sources[#idx], writer)?;
                        }),
                        Ok(false) => self.tokens.extend(quote! {
                            #TemplDisplay::display(&#value, writer)?;
                        }),
                        Err(expr) => self.tokens.extend(quote! {
                            #TemplDisplay::display(if #expr { &sources[#idx] } else { #value }, writer)?;
                        }),
                    }

                    self.static_len += 1;
                },
                Scalar::Yield => {
                    self.tokens.extend(quote! {
                        self.0.render_block_into("TourInner", &mut *writer)?;
                    });
                },
                Scalar::Render(RenderTempl { value, .. }) => match value {
                    // Either Block, just visit_stmts, or Import Aliased, render by type
                    RenderValue::Ident(id) => {
                        match shared.templ.file().resolve_id(id) {
                            AliasKind::Block(block) => self.visit_stmts(&block.stmts, shared),
                            AliasKind::Import(import) => {
                                let name = &import.alias();
                                self.tokens.extend(quote! {
                                    #TemplDisplay::display(&#name(self), &mut *writer)?;
                                });
                            }
                        }
                    },
                    // Import directly, just render by type
                    RenderValue::Path(path) => {
                        let import = shared.templ.file().import_by_path(path);
                        let name = import.alias();
                        self.tokens.extend(quote! {
                            #TemplDisplay::display(&#name(self), &mut *writer)?;
                        });
                    },
                },
                Scalar::Expr { expr, delim } => {
                    let display = display(*delim, expr);
                    let writer = writer(*delim);
                    self.tokens.extend(quote! {
                        #TemplDisplay::display(#display, #writer)?;
                    });
                },
                Scalar::Use(_) => unreachable!("use alias statement should be discarded"),
                Scalar::Item(item) => match item.as_ref() {
                    ItemTempl::Use(item) => item.to_tokens(self.tokens),
                    ItemTempl::Const(item) => item.to_tokens(self.tokens),
                },
            },
            StmtTempl::Scope(scope) => self.visit_scope(scope, shared),
        }
    }

    fn visit_scope(&mut self, scope: &Scope, shared: &Shared) {
        match scope {
            Scope::Root { stmts } => {
                token::Brace::default()
                    .surround(self.tokens, |tokens|{
                        let mut visitor = Visitor { tokens, static_len: self.static_len  };
                        visitor.visit_stmts(stmts, shared);
                        self.static_len = visitor.static_len;
                    });
            },
            Scope::If { templ, stmts, else_branch } => {
                templ.if_token.to_tokens(self.tokens);
                templ.cond.to_tokens(self.tokens);
                token::Brace::default()
                    .surround(self.tokens, |tokens|{
                        let mut visitor = Visitor { tokens, static_len: self.static_len  };
                        visitor.visit_stmts(stmts, shared);
                        self.static_len = visitor.static_len;
                    });

                if let Some((else_token, else_scope)) = else_branch {
                    else_token.to_tokens(self.tokens);
                    self.visit_scope(else_scope, shared);
                }
            },
            Scope::For { templ, stmts, else_branch } => {
                let expr = &templ.expr;

                self.tokens.extend(quote! {
                    let __for_expr = #expr;
                });

                templ.for_token.to_tokens(self.tokens);
                templ.pat.to_tokens(self.tokens);
                templ.in_token.to_tokens(self.tokens);
                format_ident!("__for_expr").to_tokens(self.tokens);

                token::Brace::default()
                    .surround(self.tokens, |tokens|{
                        let mut visitor = Visitor { tokens, static_len: self.static_len  };
                        visitor.visit_stmts(stmts, shared);
                        self.static_len = visitor.static_len;
                    });

                if let Some((_, else_scope)) = else_branch {
                    self.tokens.extend(quote! {
                        if ExactSizeIterator::len(&IntoIterator::into_iter(__for_expr)) == 0
                    });

                    self.visit_scope(else_scope, shared);
                }
            },
            Scope::Block { .. } => unreachable!("`block` scope should be replaced with `render`")
        }
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
