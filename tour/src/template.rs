use crate::{Result, TemplWrite};

/// A template renderer.
///
/// User may not implement this directly but instead use provided derive macro
/// [`Template`][tour_macros::Template].
pub trait Template {
    fn render_into(&self, render: &mut impl TemplWrite) -> Result<()>;

    fn render_block_into(&self, _block: &str, _render: &mut impl TemplWrite) -> Result<()> {
        Err(crate::Error::NoBlock)
    }

    fn render(&self) -> Result<String> {
        let (min,max) = self.size_hint();
        let mut buffer = String::with_capacity(max.unwrap_or(min));
        self.render_into(&mut buffer)?;
        Ok(buffer)
    }

    fn render_block(&self, block: &str) -> Result<String> {
        let mut buffer = String::with_capacity(128);
        self.render_block_into(block, &mut buffer)?;
        Ok(buffer)
    }

    fn size_hint(&self) -> (usize,Option<usize>) {
        (0,None)
    }
}

