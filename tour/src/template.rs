use crate::{Error, Result, TemplWrite};

/// A template content.
///
/// User may not implement this directly but instead use provided derive macro
/// [`Template`][tour_macros::Template].
pub trait Template {
    /// Render the entire content into [`writer`][TemplWrite].
    fn render_into(&self, writer: &mut impl TemplWrite) -> Result<()>;

    /// Render selected block content into [`writer`][TemplWrite].
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoBlock`] if no block found, otherwise propagete error from renderer or
    /// writer.
    fn render_block_into(&self, _block: &str, _writer: &mut impl TemplWrite) -> Result<()> {
        Err(Error::NoBlock)
    }

    /// Render the entire content into [`String`].
    fn render(&self) -> Result<String> {
        let (min,max) = self.size_hint();
        let mut buffer = String::with_capacity(max.unwrap_or(min));
        self.render_into(&mut buffer)?;
        Ok(buffer)
    }

    /// Render selected block into [`String`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoBlock`] if no block found, otherwise propagete error from renderer or
    /// writer.
    fn render_block(&self, block: &str) -> Result<String> {
        if !self.contains_block(block) {
            return Err(Error::NoBlock)
        }
        let (min,max) = self.size_hint_block(block);
        let mut buffer = String::with_capacity(max.unwrap_or(min));
        self.render_block_into(block, &mut buffer)?;
        Ok(buffer)
    }

    /// Returns `true` if given block name found.
    fn contains_block(&self, _block: &str) -> bool {
        false
    }

    /// Returns the lower and upper bounds on the rendered content length.
    fn size_hint(&self) -> (usize,Option<usize>) {
        (0,None)
    }

    /// Returns the lower and upper bounds on the selected block content length.
    fn size_hint_block(&self, _block: &str) -> (usize,Option<usize>) {
        (0,None)
    }
}

