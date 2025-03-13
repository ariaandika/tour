//! the [`Render`] trait
use crate::template::Result;

pub trait Renderer {
    /// render a buffer with escapes
    fn write_str(&mut self, value: &str) -> Result<()>;
}

impl Renderer for Vec<u8> {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.extend_from_slice(value.as_bytes());
        Ok(())
    }
}

impl Renderer for String {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.push_str(value);
        Ok(())
    }
}

pub trait Render {
    fn render(&self, f: &mut impl Renderer) -> Result<()>;
}

impl<R> Render for &R where R: Render {
    fn render(&self, f: &mut impl Renderer) -> Result<()> {
        R::render(*self, f)
    }
}

impl Render for str {
    fn render(&self, f: &mut impl Renderer) -> Result<()> {
        f.write_str(self)
    }
}

impl Render for &str {
    fn render(&self, f: &mut impl Renderer) -> Result<()> {
        f.write_str(self)
    }
}

impl Render for String {
    fn render(&self, f: &mut impl Renderer) -> Result<()> {
        f.write_str(self)
    }
}

macro_rules! render_int {
    ($t:ty) => {
        impl Render for $t {
            fn render(&self, f: &mut impl Renderer) -> Result<()> {
                f.write_str(itoa::Buffer::new().format(*self))
            }
        }
    };
}

render_int!(u8);
render_int!(u16);
render_int!(u32);
render_int!(u64);
render_int!(u128);
render_int!(usize);
render_int!(i8);
render_int!(i16);
render_int!(i32);
render_int!(i64);
render_int!(i128);
render_int!(isize);

pub struct Escape<W>(pub W);

impl<W> Renderer for Escape<W> where W: Renderer {
    fn write_str(&mut self, value: &str) -> Result<()> {
        // TODO: escape
        W::write_str(&mut self.0, value)
    }
}

