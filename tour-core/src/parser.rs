use crate::{Delimiter, ParseError, Result, visitor::Visitor};

/// Template source code parser.
///
/// For more details see the [crate level docs][crate].
pub struct Parser<'a,V> {
    source: &'a [u8],

    // parser states
    index: usize,
    state: ParseState,
    visitor: V,
}

impl<'a, V> Parser<'a, V> {
    /// Create new [`Parser`].
    ///
    /// Requires a [`Visitor`] implementation.
    ///
    /// For static content only, use [`StaticVisitor`][super::StaticVisitor].
    pub fn new(source: &'a str, visitor: V) -> Self {
        Self {
            source: source.as_bytes(),
            index: 0,
            state: ParseState::Static { start: 0 },
            visitor,
        }
    }
}

enum ParseState {
    Static { start: usize },
    Expr { start: usize, open_delim: Delimiter },
    OpenExpr { start: usize, brace: usize, },
    CloseExpr { start: usize, brace: usize, open_delim: Delimiter, close_delim: Delimiter, },
}

impl<'a,V> Parser<'a,V>
where
    V: Visitor<'a>,
{
    /// Start parsing.
    pub fn parse(mut self) -> Result<V> {
        loop {
            let current = self.index;
            let Some(byte) = self.source.get(current) else {
                break self.parse_leftover()?;
            };

            match self.state {
                ParseState::Static { start } => {
                    self.index += 1;
                    if matches!(byte,b'{') {
                        self.state = ParseState::OpenExpr { start, brace: current }
                    }
                }
                ParseState::Expr { start, open_delim } => {
                    self.index += 1;
                    if let Some(close_delim) = Delimiter::match_close(*byte) {
                        self.state = ParseState::CloseExpr {
                            start, brace: current, open_delim, close_delim,
                        }
                    }
                }
                ParseState::OpenExpr { start, brace } => {
                    match Delimiter::match_open(*byte) {
                        Some(open_delim) => {
                            self.index += 1;
                            self.state = ParseState::Expr { start: current + 1, open_delim };
                            let statics = Self::parse_str(&self.source[start..brace]);
                            if !statics.is_empty() {
                                self.visitor.visit_static(statics)?;
                            }
                        }
                        None => self.state = ParseState::Static { start }
                    }
                }
                ParseState::CloseExpr { start, brace, open_delim, close_delim } => {
                    match byte {
                        b'}' => {
                            if open_delim != close_delim {
                                return Err(ParseError::Generic(format!(
                                    "delimiter shold be same, open `{}` closed with `{}`",
                                    open_delim,close_delim,
                                )));
                            }

                            self.index += 1;
                            self.state = ParseState::Static { start: current + 1 };
                            self.visitor.visit_expr(Self::parse_str(&self.source[start..brace]), open_delim)?;
                        }
                        _ => self.state = ParseState::Expr { start, open_delim }
                    }
                }
            }
        }

        self.visitor.finish()
    }

    fn parse_leftover(&mut self) -> Result<()> {
        match self.state {
            ParseState::Static { start } | ParseState::OpenExpr { start, .. } => {
                let statics = Self::parse_str(&self.source[start..]);
                if statics.is_empty() {
                    Ok(())
                } else {
                    self.visitor.visit_static(statics)
                }
            },
            ParseState::Expr { .. } | ParseState::CloseExpr { .. } => {
                // we dont have the closing delimiter here, just bail out
                Err(ParseError::Generic("unclosed expression".to_owned()))
            },
        }
    }

    fn parse_str(source: &[u8]) -> &str {
        std::str::from_utf8(source)
            .expect("the input is string and we only check using byte char")
    }
}

