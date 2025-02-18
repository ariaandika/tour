use syn::{parse::{Parse, ParseStream}, *};
use quote::{quote, ToTokens};

pub mod flat {
    //! flat, one dimensional tokens
    use super::*;

    pub fn parse<'a>(tokens: impl Iterator<Item = crate::tokenizer::Token<'a>>) -> syn::Result<Vec<TemplStmt>> {
        let mut output = vec![];

        for token in tokens {
            match token {
                crate::tokenizer::Token::Static(val) => output.push(TemplStmt::Static(val.into())),
                crate::tokenizer::Token::Expr(val) => output.push(syn::parse_str(val)?),
            }
        }

        Ok(output)
    }

    pub struct Parser<I> {
        iter: I
    }

    impl<I> Parser<I> {
        pub fn new(iter: I) -> Self {
            Self { iter }
        }
    }

    impl<'a, I> Iterator for Parser<I>
    where
        I: Iterator<Item = crate::tokenizer::Token<'a>>,
    {
        type Item = syn::Result<TemplStmt>;

        fn next(&mut self) -> Option<Self::Item> {
            match self.iter.next()? {
                crate::tokenizer::Token::Static(val) => Some(Ok(TemplStmt::Static(val.into()))),
                crate::tokenizer::Token::Expr(val) => Some(syn::parse_str(val)),
            }
        }
    }

    /// single statement inside `{{ }}` block or a static template
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

    impl ToTokens for TemplStmt {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                TemplStmt::Static(value) => value.to_tokens(tokens),
                TemplStmt::If(templ_if) => templ_if.to_tokens(tokens),
                TemplStmt::Else(templ_else) => templ_else.to_tokens(tokens),
                TemplStmt::Match(templ_match) => templ_match.to_tokens(tokens),
                TemplStmt::Case(templ_case) => templ_case.to_tokens(tokens),
                TemplStmt::ForLoop(templ_for_loop) => templ_for_loop.to_tokens(tokens),
                TemplStmt::While(templ_while) => templ_while.to_tokens(tokens),
                TemplStmt::Loop(templ_loop) => templ_loop.to_tokens(tokens),
                TemplStmt::Break(expr_break) => expr_break.to_tokens(tokens),
                TemplStmt::Continue(expr_continue) => expr_continue.to_tokens(tokens),
                TemplStmt::Const(expr_const) => expr_const.to_tokens(tokens),
                TemplStmt::Let(expr_let) => expr_let.to_tokens(tokens),
                TemplStmt::Value(expr) => expr.to_tokens(tokens),
                TemplStmt::End(end) => end.to_tokens(tokens),
            }
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

    impl ToTokens for TemplIf {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.if_token.to_tokens(tokens);
            self.cond.to_tokens(tokens);
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

    impl ToTokens for TemplElse {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.else_token.to_tokens(tokens);
            if let Some((if_token,cond)) = &self.if_branch {
                if_token.to_tokens(tokens);
                cond.to_tokens(tokens);
            }
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

    impl ToTokens for TemplMatch {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.match_token.to_tokens(tokens);
            self.expr.to_tokens(tokens);
        }
    }

    pub struct TemplCase {
        #[allow(dead_code)]
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

    impl ToTokens for TemplCase {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.pat.to_tokens(tokens);
            if let Some((if_token,cond)) = &self.guard {
                if_token.to_tokens(tokens);
                cond.to_tokens(tokens);
            }
            tokens.extend(quote! { => });
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

    impl ToTokens for TemplForLoop {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.for_token.to_tokens(tokens);
            self.pat.to_tokens(tokens);
            self.in_token.to_tokens(tokens);
            self.expr.to_tokens(tokens);
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

    impl ToTokens for TemplWhile {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.while_token.to_tokens(tokens);
            self.cond.to_tokens(tokens);
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

    impl ToTokens for TemplLoop {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.loop_token.to_tokens(tokens);
        }
    }

    pub struct End(kw::end);

    impl Parse for End {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self(input.parse()?))
        }
    }

    impl ToTokens for End {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            self.0.to_tokens(tokens);
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



