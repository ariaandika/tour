//! parse expression in template
//!
//! this module provide parsing any possibles
//! expression through [`Parse`] implementation of [`ExprTempl`]
use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    *,
};

/// template expressions
pub enum ExprTempl {
    /// `{{ layout "layout.html" }}`
    Layout(LayoutTempl),
    /// `{{ extends "layout.html" }}`
    Extends(ExtendsTempl),
    /// `{{ yield }}`
    Yield(Token![yield]),
    /// `{{ block Body }}`
    Block(BlockTempl),
    /// `{{ endblock }}`
    Endblock(kw::endblock),
    /// `{{ render Body }}`
    Render(RenderTempl),
    /// `{{ username.get(1..6) }}`
    Expr(Box<Expr>),
    /// `{{ const NAME: &str = "deflect" }}`
    Const(ConstTempl),
    /// `{{ if admin }}`
    If(IfTempl),
    /// `{{ else if superuser }}`
    Else(ElseTempl),
    /// `{{ endif }}`
    EndIf(kw::endif),
    /// `{{ for task in tasks }}`
    For(ForTempl),
    /// `{{ endfor }}`
    EndFor(kw::endfor),
    /// `{{ use crate::TimeDisplay }}`
    Use(UseTempl),
}

impl Parse for ExprTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        match () {
            _ if input.peek(kw::layout) => input.parse().map(Self::Layout),
            _ if input.peek(kw::extends) => input.parse().map(Self::Extends),
            _ if input.peek(Token![yield]) => input.parse().map(Self::Yield),
            _ if input.peek(kw::block) => input.parse().map(Self::Block),
            _ if input.peek(Token![static]) && input.peek2(kw::block) => input.parse().map(Self::Block),
            _ if input.peek(kw::endblock) => input.parse().map(Self::Endblock),
            _ if input.peek(kw::render) => input.parse().map(Self::Render),
            _ if input.peek(Token![const]) => input.parse().map(Self::Const),
            _ if input.peek(Token![if]) => input.parse().map(Self::If),
            _ if input.peek(Token![else]) => input.parse().map(Self::Else),
            _ if input.peek(kw::endif) => input.parse().map(Self::EndIf),
            _ if input.peek(Token![for]) => input.parse().map(Self::For),
            _ if input.peek(kw::endfor) => input.parse().map(Self::EndFor),
            _ if input.peek(Token![use]) => input.parse().map(Self::Use),
            _ => input.parse().map(Self::Expr),
        }
    }
}

/// `{{ layout "layout.html" }}`
pub struct LayoutTempl {
    #[allow(dead_code)]
    pub layout_token: kw::layout,
    pub root_token: Option<kw::root>,
    pub source: LitStr,
}

/// `{{ extends "layout.html" }}`
pub struct ExtendsTempl {
    #[allow(dead_code)]
    pub extends_token: kw::extends,
    pub root_token: Option<kw::root>,
    pub source: LitStr,
}

/// `{{ block Body }}`
pub struct BlockTempl {
    pub static_token: Option<Token![static]>,
    #[allow(unused)]
    pub block_token: kw::block,
    pub name: Ident,
}

/// `{{ render Body }}`
pub struct RenderTempl {
    #[allow(dead_code)]
    pub render_token: kw::render,
    pub name: Ident,
}

/// `{{ const NAME: &str = "deflect" }}`
pub struct ConstTempl {
    pub const_token: Token![const],
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Box<Type>,
    pub eq: Token![=],
    pub expr: Box<Expr>,
    pub semi_token: Option<Token![;]>,
}

/// `{{ if admin }}`
pub struct IfTempl {
    pub if_token: Token![if],
    pub cond: Box<Expr>,
}

/// `{{ else if superuser }}`
pub struct ElseTempl {
    pub else_token: Token![else],
    pub elif_branch: Option<(Token![if],Box<Expr>)>
}

/// `{{ for task in tasks }}`
pub struct ForTempl {
    pub for_token: Token![for],
    pub pat: Box<Pat>,
    pub in_token: Token![in],
    pub expr: Box<Expr>,
}

/// `{{ use crate::TimeDisplay }}`
pub struct UseTempl {
    pub use_token: Token![use],
    pub leading_colon: Option<Token![::]>,
    pub tree: UseTree,
    pub semi_token: Option<Token![;]>,
}

impl Parse for LayoutTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            layout_token: input.parse()?,
            root_token: input.parse()?,
            source: input.parse()?,
        })
    }
}

impl Parse for ExtendsTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            extends_token: input.parse()?,
            root_token: input.parse()?,
            source: input.parse()?,
        })
    }
}

impl Parse for BlockTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            static_token: input.parse()?,
            block_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl Parse for RenderTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            render_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl Parse for ConstTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            const_token: input.parse()?,
            ident: input.call(Ident::parse_any)?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            eq: input.parse()?,
            expr: input.parse()?,
            semi_token: input.parse()?,
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
            elif_branch: match input.peek(Token![if]) {
                true => Some((input.parse()?,input.parse()?)),
                false => None,
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

impl Parse for UseTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            use_token: input.parse()?,
            leading_colon: input.parse()?,
            tree: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

mod kw {
    syn::custom_keyword!(layout);
    syn::custom_keyword!(extends);
    syn::custom_keyword!(root);
    syn::custom_keyword!(block);
    syn::custom_keyword!(render);
    syn::custom_keyword!(endblock);
    syn::custom_keyword!(endif);
    syn::custom_keyword!(endfor);
}

