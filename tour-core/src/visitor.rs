use crate::{Delimiter, Result};

/// An expression parser.
pub trait Visitor {
    /// Finished parser output.
    type Output;

    /// Collect static content.
    fn visit_static(&mut self, source: &str) -> Result<()>;

    /// Collect expression.
    fn visit_expr(&mut self, source: &str, delim: Delimiter) -> Result<()>;

    /// Visitor output.
    fn finish(self) -> Result<Self::Output>;
}

/// [`Visitor`] implementation that only collect static content.
//
// this is used in runtime template reloading
pub struct StaticVisitor;

impl Visitor for StaticVisitor {
    type Output = ();

    fn visit_static(&mut self, _: &str) -> Result<()> {
        Ok(())
    }

    fn visit_expr(&mut self, _: &str, _: Delimiter) -> Result<()> {
        Ok(())
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(())
    }
}

