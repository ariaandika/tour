//! a one dimensional template token unit
//!
//! ```html
//! <div>
//!   {{ }}
//!   {% %}
//! </div>
//! ```
use std::ops::Range;

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

        Span::range(start..end)
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
                            self.state = TokenizerState::OpenExpr { start };
                        },
                        Some(_) => {}
                        None => {
                            self.state = TokenizerState::End;
                            return Some(Token::Static(Span::range(start..current)));
                        },
                    }
                }
                TokenizerState::OpenExpr { start } => {
                    self.offset += 1;

                    match self.source.get(current) {
                        Some(&b'{') => {
                            self.state = TokenizerState::Expr;
                            return Some(Token::Static(Span::range(start..current)));
                        }
                        Some(_) => {
                            self.state = TokenizerState::Static { start };
                        }
                        None => {
                            self.state = TokenizerState::End;
                            return Some(Token::Static(Span::range(start..current)));
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
                            return Some(Token::LitStr(self.digit()));
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
    OpenExpr { start: usize },
    CloseExpr { start: usize },
    Expr,
    End,
}

