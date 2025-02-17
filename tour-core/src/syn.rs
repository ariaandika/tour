use syn::{parse::{Parse, ParseStream}, *};

pub mod flat {
    //! flat, one dimensional tokens
    #![allow(dead_code)]
    use super::*;

    pub fn parse<'a>(tokens: impl Iterator<Item = crate::tokenizer::Token<'a>>) -> syn::Result<Vec<TemplStmt>> {
        let mut output = vec![];

        for token in tokens {
            match token {
                crate::tokenizer::Token::Static(val) => output.push(TemplStmt::Static(val.into())),
                crate::tokenizer::Token::Expr(val) => output.push(tokenize_expr(val)?),
            }
        }

        Ok(output)
    }

    pub fn tokenize_expr(val: &str) -> syn::Result<TemplStmt> {
        syn::parse_str::<TemplStmt>(val)
    }

    pub enum TemplStmt {
        Static(String),

        // if branching
        If(TemplIf),
        Else(TemplElse),

        // match branching
        Match(TemplMatch),
        Case(TemplCase),

        // iterations
        ForLoop(TemplForLoop),
        While(TemplWhile),
        Loop(TemplLoop),

        // control flow
        Break(ExprBreak),
        Continue(ExprContinue),

        // declarations
        Const(ExprConst),
        Let(ExprLet),

        // renderable value
        Value(Expr),

        // termination
        End(End),
    }

    impl Parse for TemplStmt {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(match () {
                _ if input.peek(Token![if]) => Self::If(input.parse()?),
                _ if input.peek(Token![else]) => Self::Else(input.parse()?),
                _ if input.peek(Token![match]) => Self::Match(input.parse()?),
                _ if input.peek(kw::case) => Self::Case(input.parse()?),
                _ if input.peek(Token![for]) => Self::ForLoop(input.parse()?),
                _ if input.peek(Token![while]) => Self::While(input.parse()?),
                _ if input.peek(Token![loop]) => Self::Loop(input.parse()?),
                _ if input.peek(Token![break]) => Self::Break(input.parse()?),
                _ if input.peek(Token![continue]) => Self::Continue(input.parse()?),
                _ if input.peek(Token![const]) => Self::Const(input.parse()?),
                _ if input.peek(Token![let]) => Self::Let(input.parse()?),
                _ if input.peek(kw::end) => Self::End(input.parse()?),
                _ => Self::Value(input.parse()?),
            })
        }
    }

    pub struct TemplIf {
        if_token: Token![if],
        cond: Box<Expr>,
    }

    impl Parse for TemplIf {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self {
                if_token: input.parse()?,
                cond: Box::new(input.parse()?),
            })
        }
    }

    pub struct TemplElse {
        else_token: Token![else],
        if_branch: Option<(Token![if],Box<Expr>)>,
    }

    impl Parse for TemplElse {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self {
                else_token: input.parse()?,
                if_branch: match input.peek(Token![if]) {
                    true => Some((input.parse()?,input.parse()?)),
                    false => None,
                },
            })
        }
    }

    pub struct TemplMatch {
        match_token: Token![match],
        expr: Box<Expr>,
    }

    impl Parse for TemplMatch {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self {
                match_token: input.parse()?,
                expr: input.parse()?,
            })
        }
    }

    pub struct TemplCase {
        case_token: kw::case,
        pat: Pat,
        guard: Option<(Token![if],Box<Expr>)>,
    }

    impl Parse for TemplCase {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self {
                case_token: input.parse()?,
                pat: input.call(Pat::parse_multi_with_leading_vert)?,
                guard: match input.peek(Token![if]) {
                    true => Some((input.parse()?,input.parse()?)),
                    false => None,
                },
            })
        }
    }

    pub struct TemplForLoop {
        for_token: Token![for],
        pat: Box<Pat>,
        in_token: Token![in],
        expr: Box<Expr>,
    }

    impl Parse for TemplForLoop {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self {
                for_token: input.parse()?,
                pat: Box::new(input.call(Pat::parse_single)?),
                in_token: input.parse()?,
                expr: input.parse()?,
            })
        }
    }

    pub struct TemplWhile {
        while_token: Token![while],
        cond: Box<Expr>,
    }

    impl Parse for TemplWhile {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self {
                while_token: input.parse()?,
                cond: input.parse()?,
            })
        }
    }

    pub struct TemplLoop {
        loop_token: Token![loop],
    }

    impl Parse for TemplLoop {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self { loop_token: input.parse()? })
        }
    }

    pub struct End(kw::end);

    impl Parse for End {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self(input.parse()?))
        }
    }

    pub mod kw {
        syn::custom_keyword!(end);
        syn::custom_keyword!(case);
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn basic() {
            assert!(matches!(tokenize_expr("if let Some(desc) = desc"),Ok(TemplStmt::If(_))));
            assert!(matches!(tokenize_expr("case Ok(4..) | Ok(0)"),Ok(TemplStmt::Case(_))));
            assert!(matches!(tokenize_expr(" end "),Ok(TemplStmt::End(_))));
        }
    }
}



