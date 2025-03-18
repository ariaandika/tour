//! parse expression in template
//!
//! this module provide parsing any possibles
//! expression through [`Parse`] implementation of [`ExprTempl`]
use syn::{parse::{Parse, ParseStream}, *};

/// template expressions
pub enum ExprTempl {
    /// `{{ layout "layout.html" }}`
    Layout(LayoutTempl),
    /// `{{ extends "layout.html" }}`
    Extends(ExtendsTempl),
    /// `{{ yield }}`
    Yield(Token![yield]),
    /// `{{ username.get(1..6) }}`
    Expr(Expr),
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
}

impl Parse for ExprTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        match () {
            _ if input.peek(kw::layout) => input.parse().map(Self::Layout),
            _ if input.peek(kw::extends) => input.parse().map(Self::Extends),
            _ if input.peek(Token![yield]) => input.parse().map(Self::Yield),
            _ if input.peek(Token![if]) => input.parse().map(Self::If),
            _ if input.peek(Token![else]) => input.parse().map(Self::Else),
            _ if input.peek(kw::endif) => input.parse().map(Self::EndIf),
            _ if input.peek(Token![for]) => input.parse().map(Self::For),
            _ if input.peek(kw::endfor) => input.parse().map(Self::EndFor),
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

/// `{{ if admin }}`
pub struct IfTempl {
    pub if_token: Token![if],
    pub cond: Expr,
}

/// `{{ else if superuser }}`
pub struct ElseTempl {
    pub else_token: Token![else],
    pub elif_branch: Option<(Token![if],Expr)>
}

/// `{{ for task in tasks }}`
pub struct ForTempl {
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: Expr,
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
            pat: input.call(Pat::parse_single)?,
            in_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

mod kw {
    syn::custom_keyword!(layout);
    syn::custom_keyword!(extends);
    syn::custom_keyword!(root);
    syn::custom_keyword!(endif);
    syn::custom_keyword!(endfor);
}

