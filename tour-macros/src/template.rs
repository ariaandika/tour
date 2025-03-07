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
use quote::{quote, quote_spanned, ToTokens};
use syn::{ext::IdentExt, parse::{Parse, ParseStream}, *};

/// parse template macro
///
/// see [module level documentation]
///
/// [module level documentation]: self
pub struct Template {
    writer: Expr,
    #[allow(dead_code)]
    comma: Token![,],
    expr: Expr,
}

impl Parse for Template {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            writer: input.parse()?,
            comma: input.parse()?,
            expr: input.call(parse_expr)?,
        })
    }
}

impl ToTokens for Template {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Template { writer, expr, .. } = self;
        tokens.extend(quote! {{
            let writer = #writer;
            #expr
        }});
    }
}

fn parse_stmts(input: ParseStream) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];
    while !input.is_empty() {
        stmts.push(parse_stmt(input)?);
    }
    Ok(stmts)
}

fn parse_stmt(input: ParseStream) -> Result<Stmt> {
    match () {
        _ if input.peek(Token![if]) => {
            let expr = ExprIf {
                attrs: vec![],
                if_token: input.parse()?,
                cond: input.call(Expr::parse_without_eager_brace)?.into(),
                then_branch: input.call(parse_block)?,
                else_branch: if input.peek(Token![else]) {
                    Some((input.parse()?, input.call(parse_block_expr)?.into()))
                } else {
                    None
                },
            };
            Ok(Stmt::Expr(Expr::If(expr), Some(token::Semi::default())))
        },
        _ if input.peek(Token![for]) => {
            let expr = ExprForLoop {
                attrs: vec![],
                label: None,
                for_token: input.parse()?,
                pat: input.call(Pat::parse_single)?.into(),
                in_token: input.parse()?,
                expr: input.call(Expr::parse_without_eager_brace)?.into(),
                body: input.call(parse_block)?,
            };
            Ok(Stmt::Expr(Expr::ForLoop(expr), Some(token::Semi::default())))
        }
        _ if input.peek(Token![match]) => {
            let body;
            let expr = ExprMatch {
                attrs: vec![],
                match_token: input.parse()?,
                expr: input.call(Expr::parse_without_eager_brace)?.into(),
                brace_token: syn::braced!(body in input),
                arms: {
                    let mut arms = vec![];
                    let body = body;
                    while !body.is_empty() {
                        arms.push(Arm {
                            attrs: vec![],
                            pat: body.call(Pat::parse_multi_with_leading_vert)?,
                            guard: if body.peek(Token![if]) {
                                Some((body.parse()?,body.parse()?))
                            } else {
                                None
                            },
                            fat_arrow_token: body.parse()?,
                            body: body.call(parse_block_expr)?.into(),
                            comma: body.parse()?,
                        })
                    }
                    arms
                },
            };
            Ok(Stmt::Expr(Expr::Match(expr), Some(token::Semi::default())))
        }
        _ if input.peek(token::Brace) => Ok(Stmt::Expr(input.call(parse_block_expr)?, Some(token::Semi::default()))),
        _ => Ok(Stmt::Expr(input.call(parse_expr)?, Some(token::Semi::default()))),
    }
}

fn parse_block_expr(input: ParseStream) -> Result<Expr> {
    Ok(Expr::Block(ExprBlock { attrs: vec![], label: None, block: input.call(parse_block)? }))
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
    let look = input.lookahead1();

    let block = if look.peek(token::Brace) {
        let body;
        Block {
            brace_token: syn::braced!(body in input),
            stmts: body.call(parse_stmts)?,
        }
    } else if look.peek(Token![#]) {
        Block {
            brace_token: Default::default(),
            stmts: vec![input.call(parse_stmt)?],
        }
    } else {
        return Err(look.error());
    };

    Ok(block)
}

fn parse_expr(input: ParseStream) -> Result<Expr> {
    if input.peek(token::Brace) || input.peek(Token![#]) {
        return input.call(parse_block_expr);
    }

    let ident = input.parse::<Ident>()?;
    let mut attrs = vec![];
    while input.peek(Ident) {
        attrs.push(input.call(parse_attr)?);
    }

    let body = input.call(parse_block)?;

    let tag = format!("<{}",ident);
    let tag_close = format!("</{}>",ident);

    let stream = quote_spanned! {ident.span()=>{
        #Renderer::render_unescaped(writer, &#tag);
        #(#attrs)*
        #Renderer::render_unescaped(writer, &">");
        #body
        #Renderer::render_unescaped(writer, &#tag_close);
    }
    };

    Ok(syn::parse2(stream).expect("deez2"))
}

fn parse_attr(input: ParseStream) -> Result<Expr> {
    let key = input.call(Ident::parse_any)?;
    let val: Option<Expr> = if input.peek(Token![=]) {
        let _eq = input.parse::<Token![=]>()?;
        let val = if input.peek(token::Brace) {
            Expr::Block(ExprBlock {
                attrs: vec![],
                label: None,
                block: input.parse()?,
            })
        } else {
            input.parse()?
        };

        Some(syn::parse_quote!({
            #Renderer::render_unescaped(writer, &"\"");
            #Renderer::render(writer, &#val);
            #Renderer::render_unescaped(writer, &"\"");
        }))
    } else {
        None
    };

    let key_str = format!(" {key}=");

    let stream = quote_spanned! {key.span()=>{
        #Renderer::render_unescaped(writer, &#key_str);
        #val
    }
    };

    Ok(syn::parse2(stream).expect("deez"))
}

//
// Constants
//

struct Renderer;

impl ToTokens for Renderer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! { ::tour::Renderer });
    }
}

