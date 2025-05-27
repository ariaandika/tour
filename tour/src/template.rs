use crate::{Result, TemplWrite};

/// A template renderer.
///
/// User may not implement this directly but instead use provided derive macro
/// [`Template`][tour_macros::Template].
pub trait Template {
    fn render_into(&self, render: &mut impl TemplWrite) -> Result<()>;

    fn render_layout_into(&self, render: &mut impl TemplWrite) -> Result<()> {
        self.render_into(render)
    }

    fn render(&self) -> Result<String> {
        let mut buffer = String::with_capacity(128);
        self.render_into(&mut buffer)?;
        Ok(buffer)
    }

    fn render_layout(&self) -> Result<String> {
        let mut buffer = String::with_capacity(128);
        self.render_layout_into(&mut buffer)?;
        Ok(buffer)
    }
}

