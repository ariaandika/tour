//! Syntax definition for template expression.
//!
//! This only syntax definition for partial expression like `{{ if user.is_admin() }}`.
//!
//! For full ast declaration, see [`ast`][super::ast].
use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    *,
};

/// Template statments.
pub enum StmtSyn {
    // ===== Externals =====

    /// `{{ <layout | extends> <"path"> }}`
    Layout(LayoutTempl),
    /// `{{ use <"path"> as <Ident> }}`
    Use(UseTempl),
    /// `{{ render <<Ident> | "path"> [block <Ident>] }}`
    Render(RenderTempl),

    // ===== Scoped =====

    /// `{{ [pub] [static] block <Ident> }}`
    Block(BlockTempl),
    /// `{{ if <Expr> }}`
    If(IfTempl),
    /// `{{ else [if <Expr>] }}`
    Else(ElseTempl),
    /// `{{ for <Pat> in <Expr> }}`
    For(ForTempl),
    /// `{{ endblock }}`
    Endblock(kw::endblock),
    /// `{{ endif }}`
    EndIf(kw::endif),
    /// `{{ endfor }}`
    EndFor(kw::endfor),

    // ===== Internals =====

    /// `{{ yield }}`
    Yield(Token![yield]),
    /// `{{ <ItemTempl> }}`
    Item(Box<ItemTempl>),
    /// `{{ <Expr> }}`
    Expr(Box<Expr>),
}

impl Parse for StmtSyn {
    fn parse(input: ParseStream) -> Result<Self> {
        match () {
            _ if input.peek(kw::layout) => input.parse().map(Self::Layout),
            _ if input.peek(kw::extends) => input.parse().map(Self::Layout),
            _ if UseTempl::peek(input) => input.parse().map(Self::Use),
            _ if input.peek(kw::render) => input.parse().map(Self::Render),

            _ if BlockTempl::peek(input) => input.parse().map(Self::Block),
            _ if input.peek(Token![if]) => input.parse().map(Self::If),
            _ if input.peek(Token![else]) => input.parse().map(Self::Else),
            _ if input.peek(Token![for]) => input.parse().map(Self::For),
            _ if input.peek(kw::endblock) => input.parse().map(Self::Endblock),
            _ if input.peek(kw::endif) => input.parse().map(Self::EndIf),
            _ if input.peek(kw::endfor) => input.parse().map(Self::EndFor),

            _ if input.peek(Token![yield]) => input.parse().map(Self::Yield),
            _ if ItemTempl::peek(input) => input.parse().map(Self::Item),
            _ => input.parse().map(Self::Expr),
        }
    }
}

/// `{{ <layout | extends> <"path"> }}`
pub struct LayoutTempl {
    pub layout_token: kw::layout,
    pub path: LitStr,
}

/// `{{ use <"path"> as <Ident> }}`
pub struct UseTempl {
    pub use_token: Token![use],
    pub path: LitStr,
    pub as_token: Token![as],
    pub ident: Ident,
}

/// `{{ render <<Ident> | "path"> [block <Ident>] }}`
pub struct RenderTempl {
    pub render_token: kw::render,
    pub value: RenderValue,
    pub block: Option<(kw::block,Ident)>
}

/// `<Ident> | "path"`
pub enum RenderValue {
    Ident(Ident),
    Path(LitStr),
}

/// `{{ [pub] [static] block <Ident> }}`
pub struct BlockTempl {
    pub pub_token: Option<Token![pub]>,
    pub static_token: Option<Token![static]>,
    pub block_token: kw::block,
    pub name: Ident,
}

/// `{{ if <Expr> }}`
pub struct IfTempl {
    pub if_token: Token![if],
    pub cond: Box<Expr>,
}

/// `{{ else [if <Expr>] }}`
pub struct ElseTempl {
    pub else_token: Token![else],
    pub elif_branch: Option<(Token![if],Box<Expr>)>
}

/// `{{ for <Pat> in <Expr> }}`
pub struct ForTempl {
    pub for_token: Token![for],
    pub pat: Box<Pat>,
    pub in_token: Token![in],
    pub expr: Box<Expr>,
}

/// `{{ <ItemTempl> }}`
pub enum ItemTempl {
    Use(ItemUse),
    Const(ItemConst),
}

// ===== Parse implementation =====

impl Parse for LayoutTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            layout_token: match () {
                _ if input.peek(kw::layout) => input.parse()?,
                _ if input.peek(kw::extends) => kw::layout(input.parse::<kw::extends>()?.span),
                _ => unreachable!()
            },
            path: input.parse()?,
        })
    }
}

impl UseTempl {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![use]) && input.peek2(LitStr)
    }
}

impl Parse for UseTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            use_token: input.parse()?,
            path: input.parse()?,
            as_token: input.parse()?,
            ident: input.call(Ident::parse_any)?,
        })
    }
}

impl Parse for RenderTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            render_token: input.parse()?,
            value: input.parse()?,
            block: if input.peek(kw::block) {
                Some((input.parse()?,input.call(Ident::parse_any)?))
            } else {
                None
            },
        })
    }
}

impl Parse for RenderValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let look = input.lookahead1();
        match () {
            _ if look.peek(LitStr) => input.parse().map(Self::Path),
            _ if look.peek(Ident::peek_any) => input.call(Ident::parse_any).map(Self::Ident),
            _ => Err(look.error()),
        }
    }
}

impl BlockTempl {
    fn peek(input: ParseStream) -> bool {
        (input.peek(Token![pub]) && input.peek2(Token![static]) && input.peek3(kw::block)) ||
        (input.peek(Token![pub]) && input.peek2(kw::block)) ||
        (input.peek(Token![static]) && input.peek2(kw::block)) ||
        input.peek(kw::block)
    }
}

impl Parse for BlockTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            pub_token: input.parse()?,
            static_token: input.parse()?,
            block_token: input.parse()?,
            name: input.call(Ident::parse_any)?,
        })
    }
}

impl Parse for IfTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            if_token: input.parse()?,
            cond: input.parse()?,
        })
    }
}

impl Parse for ElseTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            else_token: input.parse()?,
            elif_branch: if input.peek(Token![if]) {
                Some((input.parse()?,input.parse()?))
            } else {
                None
            },
        })
    }
}

impl Parse for ForTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            for_token: input.parse()?,
            // this Pat function that is used by syn parse
            pat: Box::new(Pat::parse_multi_with_leading_vert(input)?),
            in_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ItemTempl {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![use]) ||
        input.peek(Token![const])
    }
}

impl Parse for ItemTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        let look = input.lookahead1();
        match () {
            _ if look.peek(Token![use]) => input.parse().map(Self::Use),
            _ if look.peek(Token![const]) => input.parse().map(Self::Const),
            _ => Err(look.error()),
        }
    }
}

mod kw {
    syn::custom_keyword!(layout);
    syn::custom_keyword!(extends);
    syn::custom_keyword!(block);
    syn::custom_keyword!(render);
    syn::custom_keyword!(endblock);
    syn::custom_keyword!(endif);
    syn::custom_keyword!(endfor);
}

