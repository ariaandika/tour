use syn::{parse::{Parse, ParseStream}, *};

/// expression
pub enum ExprTempl {
    /// `{{ layout "index.html" }}`
    Layout(Layout),
    /// `{{ username }}`
    Expr(Expr),
    /// `{{ unsafe body }}`
    Unsafe(UnsafeTempl),
    /// `{{ block body }}`
    Block(BlockTempl),
    /// `{{ block body }}`
    EndBlock(EndBlockTempl),
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

/// `{{ layout "index.html" }}`
pub struct Layout {
    #[allow(dead_code)]
    pub layout: kw::layout,
    pub source: LitStr,
}

/// `{{ unsafe body }}`
pub struct UnsafeTempl {
    pub unsafe_token: Token![unsafe],
    pub expr: Expr,
}

/// `{{ block body }}`
pub struct BlockTempl {
    pub block_token: kw::block,
    pub name: Ident,
}

/// `{{ endblock body }}`
pub struct EndBlockTempl {
    pub endblock_token: kw::endblock,
    pub name: Option<Ident>,
}

/// `{{ if admin }}`
pub struct IfTempl {
    pub if_token: Token![if],
    pub cond: Expr,
}

/// `{{ if admin }}`
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

impl Parse for ExprTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        match () {
            _ if input.peek(kw::layout) => input.parse().map(Self::Layout),
            _ if input.peek(Token![unsafe]) => input.parse().map(Self::Unsafe),
            _ if input.peek(kw::block) => input.parse().map(Self::Block),
            _ if input.peek(kw::endblock) => input.parse().map(Self::EndBlock),
            _ if input.peek(Token![if]) => input.parse().map(Self::If),
            _ if input.peek(Token![else]) => input.parse().map(Self::Else),
            _ if input.peek(kw::endif) => input.parse().map(Self::EndIf),
            _ if input.peek(Token![for]) => input.parse().map(Self::For),
            _ if input.peek(kw::endfor) => input.parse().map(Self::EndFor),
            _ => input.parse().map(Self::Expr),
        }
    }
}

impl Parse for Layout {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            layout: input.parse()?,
            source: input.parse()?,
        })
    }
}

impl Parse for UnsafeTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            unsafe_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl Parse for BlockTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            block_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl Parse for EndBlockTempl {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            endblock_token: input.parse()?,
            name: input.parse()?,
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
    syn::custom_keyword!(block);
    syn::custom_keyword!(endblock);
    syn::custom_keyword!(endif);
    syn::custom_keyword!(endfor);
}

