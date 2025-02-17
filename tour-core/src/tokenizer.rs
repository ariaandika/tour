//! collection of tokenizer
//!
//! `Token {{ expr }} once` = `[Static("Token "), Expr(" expr "), Static(" once")]`

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
                TokenizeState::Static(static_start) => {
                    macro_rules! next_or_complete {
                        () => {
                            match self.iter.next() {
                                Some(some) => some,
                                None => {
                                    self.state = TokenizeState::Eof;
                                    return Some(Token::Static(&self.source[static_start..]));
                                },
                            }
                        };
                    }

                    // current check
                    if matches!(self.source.get(static_start..static_start+1),Some("{")) {
                        // second check
                        let (_,ch) = next_or_complete!();

                        if matches!(ch,'{') {
                            // empty static
                            // note that '{{' is included as static on EOF
                            let (expr_start,_) = next_or_complete!();
                            self.state = TokenizeState::Expr(expr_start);
                            continue;
                        }

                        continue;
                    }

                    let (static_end,ch) = next_or_complete!();

                    if !matches!(ch,'{') {
                        continue;
                    }

                    let (_,ch) = next_or_complete!();

                    if !matches!(ch,'{') {
                        continue;
                    }

                    // found expression
                    // note that '{{' is included as static on EOF
                    let (expr_start,_) = next_or_complete!();

                    self.state = TokenizeState::Expr(expr_start);
                    return Some(Token::Static(&self.source[static_start..static_end]));
                }
                TokenizeState::Expr(expr_start) => {
                    macro_rules! next_or_complete {
                        () => { next_or_complete!(expr_start..) };
                        // this branch will exclude '}}' on EOF
                        ($end:tt) => { next_or_complete!(expr_start..$end) };
                        ($($range:tt)*) => {
                            match self.iter.next() {
                                Some(some) => some,
                                None => {
                                    self.state = TokenizeState::Eof;
                                    // exclude empty expression
                                    let expr = &self.source[$($range)*];
                                    if expr.chars().all(|e|e.is_whitespace()) {
                                        return None;
                                    }
                                    return Some(Token::Expr(expr));
                                },
                            }
                        };
                    }

                    // current check
                    if matches!(self.source.get(expr_start..expr_start+1),Some("}")) {
                        // second check
                        let (_,ch) = next_or_complete!();

                        if matches!(ch,'}') {
                            // empty expression, exclude '}}' on EOF
                            let (static_start,_) = next_or_complete!(expr_start);
                            self.state = TokenizeState::Static(static_start);
                            continue;
                        }

                        continue;
                    }

                    // first check
                    let (expr_end,ch) = next_or_complete!();

                    if !matches!(ch,'}') {
                        continue;
                    }

                    // second check
                    let (_,ch) = next_or_complete!();

                    if !matches!(ch,'}') {
                        continue;
                    }

                    // found expr closing, exclude '}}' on EOF
                    let (static_start,_) = next_or_complete!(expr_end);

                    self.state = TokenizeState::Static(static_start);

                    // exclude empty expression
                    let expr = &self.source[expr_start..expr_end];
                    if expr.chars().all(|e|e.is_whitespace()) {
                        continue;
                    }

                    return Some(Token::Expr(expr));
                }
                TokenizeState::Eof => {
                    return None;
                }
            }
        }
    }
}

#[derive(Debug)]
enum TokenizeState {
    Static(usize),
    Expr(usize),
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
        assert_eq!(tokenizer.next(),Some(Token::Expr(" expr { object } ")));
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
        assert_eq!(tokenizer.next(),Some(Token::Expr(" expr1 ")));
        assert_eq!(tokenizer.next(),Some(Token::Expr(" expr2 ")));
        assert_eq!(tokenizer.next(),None);
    }
}

