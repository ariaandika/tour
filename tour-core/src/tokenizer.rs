//! collection of tokenizer

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
                TokenizeState::Static(start) => {
                    let Some((end,ch)) = self.iter.next() else {
                        self.state = TokenizeState::TransiteStatic;
                        return Some(Token::Static(&self.source[start..]));
                    };

                    if matches!(ch,'{') {
                        self.state = TokenizeState::TransiteExpr;
                        return Some(Token::Static(&self.source[start..end]));
                    }
                }
                TokenizeState::TransiteExpr => {
                    let Some((start,ch)) = self.iter.next() else {
                        return None;
                    };

                    if matches!(ch,'}') {
                        self.state = TokenizeState::TransiteStatic;
                        continue;
                    }

                    self.state = TokenizeState::Expr(start);
                }
                TokenizeState::Expr(start) => {
                    let Some((end,ch)) = self.iter.next() else {
                        self.state = TokenizeState::TransiteStatic;
                        return Some(Token::Expr(&self.source[start..]));
                    };

                    if matches!(ch,'}') {
                        self.state = TokenizeState::TransiteStatic;
                        return Some(Token::Expr(&self.source[start..end]));
                    }
                }
                TokenizeState::TransiteStatic => {
                    let Some((start,ch)) = self.iter.next() else {
                        return None;
                    };

                    if matches!(ch,'{') {
                        self.state = TokenizeState::TransiteExpr;
                        continue;
                    }

                    self.state = TokenizeState::Static(start);
                }
            }
        }
    }
}

#[derive(Debug)]
enum TokenizeState {
    Static(usize),
    TransiteExpr,
    Expr(usize),
    TransiteStatic,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Static(&'a str),
    Expr(&'a str),
}

#[cfg(test)]
mod test {
    use super::{Tokenizer, Token};

    #[test]
    fn basic() {
        let src = "Token { expr } once";
        let mut tokenizer = Tokenizer::new(src);
        assert_eq!(tokenizer.next(),Some(Token::Static("Token ")));
        assert_eq!(tokenizer.next(),Some(Token::Expr(" expr ")));
        assert_eq!(tokenizer.next(),Some(Token::Static(" once")));
        assert_eq!(tokenizer.next(),None);
    }

    #[test]
    fn empty_expr() {
        let src = "Token {} once";
        let mut tokenizer = Tokenizer::new(src);
        assert_eq!(tokenizer.next(),Some(Token::Static("Token ")));
        assert_eq!(tokenizer.next(),Some(Token::Static(" once")));
        assert_eq!(tokenizer.next(),None);
    }

    #[test]
    fn subsequence_expr() {
        let src = "Token { expr1 }{ expr2 }";
        let mut tokenizer = Tokenizer::new(src);
        assert_eq!(tokenizer.next(),Some(Token::Static("Token ")));
        assert_eq!(tokenizer.next(),Some(Token::Expr(" expr1 ")));
        assert_eq!(tokenizer.next(),Some(Token::Expr(" expr2 ")));
        assert_eq!(tokenizer.next(),None);
    }
}

