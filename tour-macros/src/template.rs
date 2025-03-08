//! embeded template
//!
//! ```no_run
//! render!(&mut writer, {
//!     section id="title" {
//!         h1 class="text-4xl font-bold" # "Orders";
//!     }
//!
//!     if let Some(note) = note {
//!         div id="note" # note;
//!     }
//!
//!     for order in orders {
//!         div id={order.id} # &order.name;
//!     } else {
//!         div # "no orders";
//!     }
//!
//!     match &selected.state {
//!         State::Ongoing => div # "your order is ongoing",
//!         State::Arrive => div # "your order has arrived",
//!     }
//! });
//! ```
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{ext::IdentExt, parse::{Parse, ParseStream}, *};

/// parse template macro
///
/// see [module level documentation]
///
/// [module level documentation]: self
pub struct Template {
    writer: Expr,
    stmts: Vec<Stmt>,
}

impl Parse for Template {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            writer: input.parse()?,
            stmts: {
                input.parse::<Token![,]>()?;
                input.call(parse_stmts)?
            },
        })
    }
}

impl ToTokens for Template {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Template { writer, stmts, .. } = self;
        let mut body = quote! {
            let writer = #writer;
            let __render = ::tour::Renderer::render;
            let __render_unsafe = ::tour::Renderer::render_unescaped;
        };
        quote! {#(#stmts)*}.to_tokens(&mut body);
        token::Brace::default().surround(tokens, |t|*t=body);
    }
}

fn parse_stmts(input: ParseStream) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];
    while !input.is_empty() {
        stmts.push(input.call(parse_stmt)?);
    }
    Ok(stmts)
}

fn parse_stmt(input: ParseStream) -> Result<Stmt> {
    Ok(match () {
        _ if input.peek(Token![let]) => Stmt::Local(Local {
            attrs: input.call(Attribute::parse_inner)?,
            let_token: input.parse()?,
            pat: input.call(Pat::parse_single)?,
            init: if input.peek(Token![=]) {
                Some(LocalInit {
                    eq_token: input.parse()?,
                    expr: input.call(Expr::parse_without_eager_brace)?.into(),
                    diverge: if input.peek(Token![else]) {
                        Some((input.parse()?, input.parse()?))
                    } else {
                        None
                    },
                })
            } else {
                None
            },
            semi_token: input.parse()?,
        }),
        _ if input.peek(Token![#]) => {
            input.parse::<Token![#]>()?;
            let expr = input.parse::<Expr>()?;
            Stmt::Expr(
                syn::parse_quote!(__render(writer, &#expr)),
                input.parse()?
            )
        },
        _ if input.peek2(Token![!]) => Stmt::Macro(StmtMacro {
            attrs: input.call(Attribute::parse_inner)?,
            mac: input.parse()?,
            semi_token: input.parse()?,
        }),
        _ => Stmt::Expr(input.call(parse_expr)?, input.parse()?),
    })
}

fn parse_expr(input: ParseStream) -> Result<Expr> {
    Ok(match input.lookahead1() {
        look if look.peek(Token![if]) => Expr::If(ExprIf {
            attrs: input.call(Attribute::parse_inner)?,
            if_token: input.parse()?,
            cond: input.call(Expr::parse_without_eager_brace)?.into(),
            then_branch: input.call(parse_block)?,
            else_branch: if input.peek(Token![else]) {
                Some((input.parse()?, Expr::Block(ExprBlock {
                    attrs: input.call(Attribute::parse_inner)?,
                    label: input.parse()?,
                    block: input.call(parse_block)?,
                }).into()))
            } else {
                None
            },
        }),
        look if look.peek(Token![for]) => Expr::ForLoop(ExprForLoop {
            attrs: input.call(Attribute::parse_inner)?,
            label: input.parse()?,
            for_token: input.parse()?,
            pat: input.call(Pat::parse_single)?.into(),
            in_token: input.parse()?,
            expr: input.call(Expr::parse_without_eager_brace)?.into(),
            body: input.call(parse_block)?,
        }),
        look if look.peek(Token![continue]) || look.peek(Token![break]) => {
            let expr = input.parse::<Expr>()?;
            input.parse::<Option<Token![;]>>()?;
            expr
        }
        look if look.peek(Token![match]) => {
            let body;
            Expr::Match(ExprMatch {
                attrs: input.call(Attribute::parse_inner)?,
                match_token: input.parse()?,
                expr: input.call(Expr::parse_without_eager_brace)?.into(),
                brace_token: syn::braced!(body in input),
                arms: {
                    let mut arms = vec![];
                    let input = body;
                    while !input.is_empty() {
                        arms.push(Arm {
                            attrs: input.call(Attribute::parse_inner)?,
                            pat: input.call(Pat::parse_multi_with_leading_vert)?,
                            guard: if input.peek(Token![if]) {
                                Some((input.parse()?, input.call(Expr::parse_without_eager_brace)?.into()))
                            } else {
                                None
                            },
                            fat_arrow_token: input.parse()?,
                            body: input.call(parse_expr)?.into(),
                            comma: input.parse()?,
                        })
                    }
                    arms
                },
            })
        }
        look if look.peek(token::Brace) => Expr::Block(ExprBlock {
            attrs: input.call(Attribute::parse_inner)?,
            label: input.parse()?,
            block: input.call(parse_block)?,
        }),
        look if look.peek(Ident::peek_any) => {
            let ident = input.parse::<Ident>()?;
            let mut attrs = vec![];
            while input.peek(Ident::peek_any) {
                attrs.push(input.call(parse_attr)?);
            }
            let body = input.call(parse_block)?;

            let tag = format!("<{}",ident);
            let tag_close = format!("</{}>",ident);

            let stmts = vec![
                syn::parse_quote!(__render_unsafe(writer, &#tag);),
                syn::parse_quote!(#(#attrs)*),
                syn::parse_quote!(__render_unsafe(writer, &">");),
                syn::parse_quote!(#body),
                syn::parse_quote!(__render_unsafe(writer, &#tag_close);),
            ];

            Expr::Block(ExprBlock {
                attrs: vec![], label: None,
                block: Block {
                    brace_token: Default::default(),
                    stmts,
                }
            })
        }
        look => return Err(look.error()),
    })
}

/// a block that may contains template
///
/// ```no_run
/// // braced
/// template! {
///     if assert {
///         // this block
///         div id="section";
///     }
/// }
/// // inlined
/// template! {
///     if assert # div id="section";
/// }
/// ```
fn parse_block(input: ParseStream) -> Result<Block> {
    Ok(match input.lookahead1() {
        look if look.peek(token::Brace) => {
            let body;
            Block {
                brace_token: syn::braced!(body in input),
                stmts: body.call(parse_stmts)?,
            }
        }
        look if look.peek(Token![#]) => {
            input.parse::<Token![#]>()?;
            if input.peek(token::Brace) {
                let body;
                Block {
                    brace_token: syn::braced!(body in input),
                    stmts: {
                        let expr = Stmt::Expr(body.parse()?, body.parse()?);
                        vec![syn::parse_quote!(__render(writer, &#expr);)]
                    },
                }
            } else {
                Block {
                    brace_token: Default::default(),
                    stmts: {
                        let expr = input.call(parse_expr)?;
                        vec![Stmt::Expr(expr, input.parse()?)]
                    },
                }
            }
        }
        look if look.peek(Token![;]) => {
            input.parse::<Token![;]>()?;
            Block {
                brace_token: Default::default(),
                stmts: vec![],
            }
        }
        look => return Err(look.error()),
    })
}

fn parse_attr(input: ParseStream) -> Result<Expr> {
    let key = input.call(Ident::parse_any)?.to_string();

    if input.parse::<Option<Token![=]>>()?.is_none() {
        return Ok(syn::parse_quote!(__render_unsafe(writer, #key);))
    }

    let val = input.call(Expr::parse_without_eager_brace)?;
    let key_str = format!(" {key}=\"");

    Ok(syn::parse_quote!({
        __render_unsafe(writer, &#key_str);
        __render(writer, &#val);
        __render_unsafe(writer, &"\"");
    }))
}

