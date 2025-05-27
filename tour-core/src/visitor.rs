use crate::{Delimiter, Result};

/// An expression parser.
pub trait ExprParser {
    /// Finished parser output.
    type Output;

    /// Collect static content.
    fn collect_static(&mut self, source: &str) -> Result<()>;

    /// Collect expression.
    fn parse_expr(&mut self, source: &str, delim: Delimiter) -> Result<()>;

    /// parser output before consumed in codegen
    fn finish(self) -> Result<Self::Output>;
}

/// [`ExprParser`] implementation that does nothing.
//
// this is used in runtime template reloading
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

