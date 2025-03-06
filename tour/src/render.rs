//! the [`Render`] trait

pub trait Renderer {
    /// render a buffer with escapes
    fn render(&mut self, buf: &impl Render);
    /// render a buffer without escapes
    fn render_unescaped(&mut self, buf: &impl Render) {
        self.render(buf);
    }
}

impl Renderer for String {
    fn render(&mut self, buf: &impl Render) {
        self.push_str(unsafe { std::str::from_utf8_unchecked(buf.as_bytes()) });
    }
}

pub trait Render {
    fn render_to(&self, writer: &mut impl Renderer);
    fn as_bytes(&self) -> &[u8];
}

impl<R> Render for &R where R: Render {
    fn render_to(&self, _: &mut impl Renderer) {
        todo!()
    }

    fn as_bytes(&self) -> &[u8] {
        R::as_bytes(*self)
    }
}

impl Render for &str {
    fn render_to(&self, _: &mut impl Renderer) {
        todo!()
    }

    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(*self)
    }
}

