//! collection of tokenizer
//!
//! ```
//! use tour_core::tokenizer::{Tokenizer, Token};
//! let src = "Token {{ expr { object } }} once { ignored }";
//! let mut tokenizer = Tokenizer::new(src);
//! assert_eq!(tokenizer.next(),Some(Token::Static("Token ")));
//! assert_eq!(tokenizer.next(),Some(Token::Expr("expr { object }")));
//! assert_eq!(tokenizer.next(),Some(Token::Static(" once { ignored }")));
//! assert_eq!(tokenizer.next(),None);
//! ```

/// a tokenizer where the source ownership is hold by the caller
pub struct Tokenizer<'a> {
    source: &'a str,
    state: TokenizeState,
    iter: std::str::CharIndices<'a>,
}

impl<'a> Tokenizer<'a> {
    /// create new [`Tokenizer`]
    pub fn new(source: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            source,
            state: TokenizeState::Static(0),
            iter: source.char_indices(),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                TokenizeState::Static(start) => match self.iter.next() {
                    Some((_,'{')) => self.state = TokenizeState::OpenExpr(start),
                    Some(_) => { },
                    None => {
                        let content = &self.source[start..];
                        return if content.is_empty() {
                            None
                        } else {
                            self.state = TokenizeState::Eof;
                            Some(Token::Static(content))
                        }
                    },
                },
                TokenizeState::Expr(start) => match self.iter.next() {
                    Some((_,'}')) => self.state = TokenizeState::CloseExpr(start),
                    Some(_) => { },
                    None => {
                        let expr = &self.source[start..];
                        return if expr.is_empty() {
                            None
                        } else {
                            self.state = TokenizeState::Eof;
                            Some(Token::Expr(expr))
                        }
                    },
                },
                TokenizeState::OpenExpr(start) => match self.iter.next() {
                    Some((start_expr,'{')) => {
                        self.state = TokenizeState::StartExpr;
                        let content = &self.source[start..start_expr - 1];
                        if !content.is_empty() {
                            return Some(Token::Static(content))
                        }
                    },
                    Some(_) => {
                        self.state = TokenizeState::Static(start);
                    },
                    None => {
                        let content = &self.source[start..];
                        return if content.is_empty() {
                            None
                        } else {
                            self.state = TokenizeState::Eof;
                            Some(Token::Static(content))
                        }
                    },
                },
                TokenizeState::CloseExpr(start) => match self.iter.next() {
                    Some((start_static,'}')) => {
                        self.state = TokenizeState::EndExpr;
                        let content = self.source[start..start_static - 1].trim();
                        if !content.is_empty() {
                            return Some(Token::Expr(content))
                        }
                    },
                    Some(_) => {
                        self.state = TokenizeState::Expr(start);
                    },
                    None => {
                        let content = self.source[start..].trim();
                        return if content.is_empty() {
                            None
                        } else {
                            self.state = TokenizeState::Eof;
                            Some(Token::Expr(content))
                        }
                    },
                },
                TokenizeState::StartExpr => self.state = match self.iter.next()? {
                    (n,'}') => TokenizeState::CloseExpr(n),
                    (n,_) => TokenizeState::Expr(n)
                },
                TokenizeState::EndExpr => self.state = match self.iter.next()? {
                    (n,'{') => TokenizeState::OpenExpr(n),
                    (n,_) => TokenizeState::Static(n)
                },
                TokenizeState::Eof => return None,
            }
        }
    }
}

#[derive(Debug)]
pub enum TokenizeState {
    /// last item is a static value
    Static(usize),
    /// last item is a '{'
    OpenExpr(usize),
    /// state after [`TokenizeState::OpenExpr`] which the index still point to '{'
    StartExpr,
    /// last item is an expression
    Expr(usize),
    /// last item is a '}'
    CloseExpr(usize),
    /// state after [`TokenizeState::CloseExpr`] which the index still point to '}'
    EndExpr,
    /// end of iterator
    Eof,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Static(&'a str),
    Expr(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenOwned {
    Static(String),
    Expr(String),
}

impl From<Token<'_>> for TokenOwned {
    fn from(value: Token) -> Self {
        match value {
            Token::Static(val) => Self::Static(val.into()),
            Token::Expr(val) => Self::Expr(val.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Tokenizer, Token};

    #[test]
    fn basic() {
        let src = "Token {{ expr { object } }} once { ignored }";
        let mut tokenizer = Tokenizer::new(src);
        assert_eq!(tokenizer.next(),Some(Token::Static("Token ")));
        assert_eq!(tokenizer.next(),Some(Token::Expr("expr { object }")));
        assert_eq!(tokenizer.next(),Some(Token::Static(" once { ignored }")));
        assert_eq!(tokenizer.next(),None);
    }

    #[test]
    fn empty_expr() {
        let src = "Token {{}} once {{  \n }}";
        let mut tokenizer = Tokenizer::new(src);
        assert_eq!(tokenizer.next(),Some(Token::Static("Token ")));
        assert_eq!(tokenizer.next(),Some(Token::Static(" once ")));
        assert_eq!(tokenizer.next(),None);
    }

    #[test]
    fn empty_static() {
        let src = "Token {{ expr1 }}{{ expr2 }}";
        let mut tokenizer = Tokenizer::new(src);
        assert_eq!(tokenizer.next(),Some(Token::Static("Token ")));
        assert_eq!(tokenizer.next(),Some(Token::Expr("expr1")));
        assert_eq!(tokenizer.next(),Some(Token::Expr("expr2")));
        assert_eq!(tokenizer.next(),None);
    }
}

