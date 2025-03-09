use std::ops::Range;


/// source map of a token
pub struct Span {
    range: Range<usize>,
}

impl Span {
    pub fn eval<'a>(&self, source: &'a str) -> &'a str {
        &source[self.range.clone()]
    }
    pub(crate) fn range(range: Range<usize>) -> Self {
        Self { range }
    }
    pub(crate) fn offset(offset: usize) -> Self {
        Self { range: offset..offset + 1 }
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
