//! a one dimensional template token unit
//!
//! variable:
//!
//! ```html
//! {{ title }}
//! ```
//!
//! layout:
//!
//! ```html
//! {{ layout "index.html" }}
//! ```
use std::ops::Range;

/// token emited by [`Tokenizer`]
pub enum Token {
    /// static template content
    Static(Span),
    /// word identifier
    Ident(Span),
    /// identifier
    Punct(Span),
    /// literal string
    LitStr(Span),
    /// literal number
    LitNum(Span),
}

impl Token {
    fn span(&self) -> &Span {
        match self {
            Token::Static(span) |
            Token::Ident(span) |
            Token::Punct(span) |
            Token::LitStr(span) |
            Token::LitNum(span) => span
        }
    }
}

/// source map of a token
pub struct Span {
    range: Range<usize>,
}

impl Span {
    pub fn eval<'a>(&self, source: &'a str) -> &'a str {
        &source[self.range.clone()]
    }
    fn range(range: Range<usize>) -> Self {
        Self { range }
    }
    fn offset(offset: usize) -> Self {
        Self { range: offset..offset + 1 }
    }
}

pub struct Tokenizer<'a> {
    /// raw source code
    source: &'a [u8],
    offset: usize,
    state: TokenizerState
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            offset: 0,
            state: TokenizerState::Static { start: 0 },
        }
    }

    /// collect Identifier
    ///
    /// the caller must ensure that current offset is an ascii alphabatic or `_`
    /// otherwise it will panic in debug mode
    fn identifier(&mut self) -> Span {
        let start = self.offset;
        debug_assert!(matches!(self.source[start],e if e.is_ascii_alphabetic() || e == b'_'));

        self.offset += 1;

        loop {
            let current = self.offset;
            match self.source.get(current) {
                Some(ch) if
                    ch.is_ascii_alphanumeric() ||
                    ch == &b'_'
                => {
                    self.offset += 1;
                }
                _ => break,
            }
        };

        Span::range(start..self.offset)
    }

    /// collect literal string
    ///
    /// the caller must ensure that current offset is double quote `"`
    /// otherwise it will panic in debug mode
    fn litstr(&mut self) -> Span {
        let start = self.offset;
        debug_assert!(self.source[start] == b'"');

        self.offset += 1;

        let end = loop {
            let current = self.offset;
            match self.source.get(current) {
                Some(&b'"') => {
                    self.offset += 1;
                    break self.offset;
                },
                Some(_) => {
                    self.offset += 1;
                }
                None => break current,
            }
        };

        Span::range(start + 1..end - 1)
    }

    /// collect literal number
    ///
    /// the caller must ensure that current offset is an ascii digit
    /// otherwise it will panic in debug mode
    fn digit(&mut self) -> Span {
        let start = self.offset;
        debug_assert!(self.source[start].is_ascii_digit());

        self.offset += 1;

        loop {
            let current = self.offset;
            match self.source.get(current) {
                Some(ch) if
                    ch.is_ascii_digit() ||
                    ch == &b'.'
                => {
                    self.offset += 1;
                }
                _ => break,
            }
        }

        Span::range(start..self.offset)
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current = self.offset;

            match self.state {
                TokenizerState::Static { start } => {
                    self.offset += 1;

                    match self.source.get(current) {
                        Some(&b'{') => {
                            self.state = TokenizerState::OpenExpr { start, brace: current };
                        }
                        Some(_) => {}
                        None => {
                            self.state = TokenizerState::End;
                            let range = start..current;
                            if !range.is_empty() {
                                return Some(Token::Static(Span::range(range)));
                            }
                        },
                    }
                }
                TokenizerState::OpenExpr { start, brace } => {
                    self.offset += 1;

                    match self.source.get(current) {
                        Some(&b'{') => {
                            self.state = TokenizerState::Expr;
                            let range = start..brace;
                            if !range.is_empty() {
                                return Some(Token::Static(Span::range(range)));
                            }
                        }
                        Some(_) => {
                            self.state = TokenizerState::Static { start };
                        }
                        None => {
                            self.state = TokenizerState::End;
                            let range = start..current;
                            if !range.is_empty() {
                                return Some(Token::Static(Span::range(range)));
                            }
                        }
                    }
                }
                TokenizerState::Expr => {
                    match self.source.get(current) {
                        Some(&b'}') => {
                            self.offset += 1;
                            self.state = TokenizerState::CloseExpr { start: current };
                        }
                        Some(&b'"') => {
                            return Some(Token::LitStr(self.litstr()));
                        }
                        Some(next) if next.is_ascii_whitespace() => {
                            self.offset += 1;
                        }
                        Some(next) if next.is_ascii_alphabetic() || next == &b'_' => {
                            return Some(Token::Ident(self.identifier()));
                        }
                        Some(next) if next.is_ascii_digit() => {
                            return Some(Token::LitNum(self.digit()));
                        }
                        Some(next) if next.is_ascii_punctuation() => {
                            self.offset += 1;
                            return Some(Token::Punct(Span::offset(current)));
                        }
                        Some(_) => {
                            // NOTE: invalid character is skipped
                            self.offset += 1;
                        }
                        None => {
                            self.state = TokenizerState::End;
                            return None;
                        }
                    }
                }
                TokenizerState::CloseExpr { start } => {
                    match self.source.get(current) {
                        Some(&b'}') => {
                            self.offset += 1;
                            self.state = TokenizerState::Static { start: self.offset };
                        },
                        Some(_) => {
                            self.state = TokenizerState::Expr;
                            return Some(Token::Punct(Span::offset(start)));
                        }
                        None => {
                            self.state = TokenizerState::End;
                            return Some(Token::Punct(Span::offset(start)));
                        }
                    }
                }
                TokenizerState::End => return None,
            }
        }
    }
}

enum TokenizerState {
    Static { start: usize },
    OpenExpr { start: usize, brace: usize },
    CloseExpr { start: usize },
    Expr,
    End,
}

mod impls {
    use super::*;

    impl PartialEq<Span> for Token {
        fn eq(&self, other: &Span) -> bool {
            self.span() == other
        }
    }

    impl PartialEq for Span {
        fn eq(&self, other: &Self) -> bool {
            &self.range == &other.range
        }
    }

    impl PartialEq<Range<usize>> for Span {
        fn eq(&self, other: &Range<usize>) -> bool {
            &self.range == other
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Tokenizer, Token};

    macro_rules! assert_next {
        ($tokens:ident) => {
            assert!(matches!($tokens.next(),None))
        };
        ($tokens:ident,$src:ident,$variant:ident,$expect:literal) => {
            let next = $tokens.next().unwrap();
            assert!(matches!(next,Token::$variant(_)));
            assert_eq!(next.span().eval($src),$expect);
        };
    }

    #[test]
    fn basic() {
        let src = "Token {{ expr { object } }} once { ignored }";
        let mut tokens = Tokenizer::new(src);

        assert_next!(tokens,src,Static,"Token ");
        assert_next!(tokens,src,Ident,"expr");
        assert_next!(tokens,src,Punct,"{");
        assert_next!(tokens,src,Ident,"object");
        assert_next!(tokens,src,Punct,"}");
        assert_next!(tokens,src,Static," once { ignored }");
        assert_next!(tokens);
    }

    #[test]
    fn lit_str() {
        let src = r#"Token {{ include "layout.html" }}"#;
        let mut tokens = Tokenizer::new(src);

        assert_next!(tokens,src,Static,"Token ");
        assert_next!(tokens,src,Ident,"include");
        assert_next!(tokens,src,LitStr,"layout.html");
        assert_next!(tokens);
    }

    #[test]
    fn empty_expr() {
        let src = "Token {{}} once {{  \n }}";
        let mut tokens = Tokenizer::new(src);

        assert_next!(tokens,src,Static,"Token ");
        assert_next!(tokens,src,Static," once ");
        assert_next!(tokens);
    }

    #[test]
    fn empty_static() {
        let src = "Token {{ expr1 }}{{ expr2 }}";
        let mut tokens = Tokenizer::new(src);

        assert_next!(tokens,src,Static,"Token ");
        assert_next!(tokens,src,Ident,"expr1");
        assert_next!(tokens,src,Ident,"expr2");
        assert_next!(tokens);
    }
}

