use syn::{parse::{Parse, ParseStream}, *};

/// expression
#[derive(Debug)]
pub enum ExprTempl {
    /// `{{ extends "index.html" }}`
    Extends(Extends),
    /// `{{ username }}`
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

/// `{{ extends "index.html" }}`
#[derive(Debug)]
pub struct Extends {
    #[allow(dead_code)]
    pub extends: kw::extends,
    pub source: LitStr,
}

/// `{{ if admin }}`
#[derive(Debug)]
pub struct IfTempl {
    pub if_token: Token![if],
    pub cond: Expr,
}

/// `{{ if admin }}`
#[derive(Debug)]
pub struct ElseTempl {
    pub else_token: Token![else],
    pub elif_branch: Option<(Token![if],Expr)>
}

/// `{{ for task in tasks }}`
#[derive(Debug)]
pub struct ForTempl {
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: Expr,
}

impl Parse for ExprTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        match () {
            _ if input.peek(kw::extends) => input.parse().map(Self::Extends),
            _ if input.peek(Token![if]) => input.parse().map(Self::If),
            _ if input.peek(Token![else]) => input.parse().map(Self::Else),
            _ if input.peek(kw::endif) => input.parse().map(Self::EndIf),
            _ if input.peek(Token![for]) => input.parse().map(Self::For),
            _ if input.peek(kw::endfor) => input.parse().map(Self::EndFor),
            _ => input.parse().map(Self::Expr),
        }
    }
}

impl Parse for Extends {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            extends: input.parse()?,
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
    syn::custom_keyword!(extends);
    syn::custom_keyword!(endif);
    syn::custom_keyword!(endfor);
}

