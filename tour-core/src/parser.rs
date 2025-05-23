//! attempt to split the expression parser as trait
//! to prevent dependencies on syn and friends
use crate::Delimiter;

/// a template expression parser
pub trait ExprParser {
    /// finished parser output
    ///
    /// this will be consumed in codegen after parsing
    type Output;

    /// save static content
    fn collect_static(&mut self, source: &str) -> Result<()>;

    /// parse expression
    fn parse_expr(&mut self, source: &str, delim: Delimiter) -> Result<()>;

    /// parser output before consumed in codegen
    fn finish(self) -> Result<Self::Output>;
}

/// parse output
///
/// template then can be generated to static source code at compile time
/// or static content at runtime
pub struct Template<'a, E> {
    /// expression parser output
    pub output: E,
    /// static contents
    pub statics: Vec<&'a str>
}

enum ParseState {
    Static { start: usize },
    Expr { start: usize, open_delim: Delimiter },
    OpenExpr { start: usize, brace: usize, },
    CloseExpr { start: usize, brace: usize, open_delim: Delimiter, close_delim: Delimiter, },
}

/// template source code parser
pub struct Parser<'a,E> {
    source: &'a [u8],

    // parser states
    index: usize,
    state: ParseState,
    expr: E,

    statics: Vec<&'a str>,
}

impl<'a,E> Parser<'a,E> {
    /// create new parser
    ///
    /// it requires an expression parser
    ///
    /// for static content only, use [`NoopParser`]
    pub fn new(source: &'a str, expr_parser: E) -> Self {
        Self {
            source: source.as_bytes(),
            index: 0,
            state: ParseState::Static { start: 0 },
            expr: expr_parser,
            statics: vec![],
        }
    }
}

impl<'a,E> Parser<'a,E>
where
    E: ExprParser,
{
    /// start parsing
    pub fn parse(mut self) -> Result<Template<'a,E::Output>> {
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
                            self.collect_static(&self.source[start..brace])?;
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
                            self.parse_expr(&self.source[start..brace],open_delim)?;
                        }
                        _ => self.state = ParseState::Expr { start, open_delim }
                    }
                }
            }
        }

        Ok(Template {
            output: self.expr.finish()?,
            statics: self.statics,
        })
    }

    fn parse_leftover(&mut self) -> Result<()> {
        match self.state {
            ParseState::Static { start } | ParseState::OpenExpr { start, .. } => {
                self.collect_static(&self.source[start..])
            }
            ParseState::Expr { .. } | ParseState::CloseExpr { .. } => {
                // we dont have the closing delimiter here, just bail out
                Err(ParseError::Generic("unclosed expression".to_owned()))
            }
        }
    }

    fn collect_static(&mut self, source: &'a [u8]) -> Result<()> {
        if source.is_empty() {
            return Ok(())
        }

        let source = Self::parse_str(source);
        self.statics.push(source);
        self.expr.collect_static(source)?;

        Ok(())
    }

    fn parse_expr(&mut self, source: &[u8], delim: Delimiter) -> Result<()> {
        self.expr.parse_expr(Self::parse_str(source), delim)
    }

    fn parse_str(source: &[u8]) -> &str {
        std::str::from_utf8(source)
            .expect("the input is string and we only check using byte char")
    }
}

/// an [`ExprParser`] that does nothing
///
/// this used in runtime template reloading
pub struct NoopParser;

impl ExprParser for NoopParser {
    type Output = ();

    fn collect_static(&mut self, _: &str) -> Result<()> {
        Ok(())
    }

    fn parse_expr(&mut self, _: &str, _: Delimiter) -> Result<()> {
        Ok(())
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(())
    }
}

/// result alias for [`ParseError`]
pub type Result<T,E = ParseError> = core::result::Result<T,E>;

/// an error that may occur during parsing in [`Parser`]
#[derive(Debug)]
pub enum ParseError {
    Generic(String),
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Generic(s) => f.write_str(s),
        }
    }
}

