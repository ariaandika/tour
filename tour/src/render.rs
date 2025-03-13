//! the [`TourDisplay`] trait
use crate::template::Result;

pub trait Writer {
    /// render a buffer with escapes
    fn write_str(&mut self, value: &str) -> Result<()>;
}

impl<R> Writer for &mut R where R: Writer {
    fn write_str(&mut self, value: &str) -> Result<()> {
        R::write_str(self, value)
    }
}

impl Writer for Vec<u8> {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.extend_from_slice(value.as_bytes());
        Ok(())
    }
}

impl Writer for String {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.push_str(value);
        Ok(())
    }
}

pub trait Display {
    fn display(&self, f: &mut impl Writer) -> Result<()>;
}

impl<R> Display for &R where R: Display {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        R::display(*self, f)
    }
}

impl Display for str {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        f.write_str(self)
    }
}

impl Display for &str {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        f.write_str(self)
    }
}

impl Display for String {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        f.write_str(self)
    }
}

macro_rules! render_int {
    ($t:ty) => {
        impl Display for $t {
            fn display(&self, f: &mut impl Writer) -> Result<()> {
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

/// wrap Renderer to escape input
///
/// escape based on [OWASP recommendation](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
pub struct Escape<W>(pub W);

impl<W> Writer for Escape<W> where W: Writer {
    fn write_str(&mut self, value: &str) -> Result<()> {
        let mut latest = 0;
        let mut iter = value.char_indices();

        loop {
            let Some((i,ch)) = iter.next() else {
                break;
            };

            let escaped = match ch {
                '&' => "&amp",
                '<' => "&lt",
                '>' => "&gt",
                '"' => "&quot",
                '\'' => "&#x27",
                _ => continue,
            };

            self.0.write_str(&value[latest..i])?;
            self.0.write_str(escaped)?;

            latest = i + 1;
        }

        if let Some(value) = value.get(latest..) {
            if !value.is_empty() {
                self.0.write_str(value)?;
            }
        }

        Ok(())
    }
}

