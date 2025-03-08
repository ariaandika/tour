//! the [`Render`] trait

pub trait Renderer {
    /// render a buffer with escapes
    fn render<S>(&mut self, value: S)
    where
        S: AsRef<str>;
    /// render a buffer without escapes
    fn render_unescaped<S>(&mut self, value: S)
    where
        S: AsRef<str>
    {
        self.render(value);
    }
}

impl Renderer for Vec<u8> {
    fn render<S>(&mut self, value: S)
    where
        S: AsRef<str>
    {
        self.extend_from_slice(value.as_ref().as_bytes());
    }
}

impl Renderer for String {
    fn render<S>(&mut self, value: S)
    where
        S: AsRef<str>
    {
        self.push_str(value.as_ref());
    }
}

