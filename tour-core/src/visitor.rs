use std::borrow::Cow;

use crate::{Delimiter, Result};

/// An expression parser.
pub trait Visitor<'a>: Sized {
    /// Collect static content.
    fn visit_static(&mut self, source: &'a str) -> Result<()>;

    /// Collect expression.
    fn visit_expr(&mut self, source: &'a str, delim: Delimiter) -> Result<()>;

    /// Final check on finish parsing.
    fn finish(self) -> Result<Self>;
}

/// [`Visitor`] implementation that only collect static content.
//
// this is used in runtime template reloading
pub struct StaticVisitor<'a> {
    pub statics: Vec<Cow<'a,str>>
}

impl StaticVisitor<'_> {
    /// Create new [`StaticVisitor`].
    pub fn new() -> Self {
        Self { statics: vec![] }
    }

    /// Convert statics into owned [`String`].
    pub fn into_owned(self) -> StaticVisitor<'static> {
        StaticVisitor {
            statics: self
                .statics
                .into_iter()
                .map(|e| Cow::Owned(e.into_owned()))
                .collect(),
        }
    }
}

impl<'a> Visitor<'a> for StaticVisitor<'a> {
    fn visit_static(&mut self, source: &'a str) -> Result<()> {
        self.statics.push(Cow::Borrowed(source));
        Ok(())
    }

    fn visit_expr(&mut self, _: &'a str, _: Delimiter) -> Result<()> {
        Ok(())
    }

    fn finish(self) -> Result<Self> {
        Ok(self)
    }
}

impl Default for StaticVisitor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

