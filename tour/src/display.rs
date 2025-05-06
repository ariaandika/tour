//! The [`Display`] trait.
use crate::{Result, Writer};

pub trait Display {
    fn display(&self, f: &mut impl Writer) -> Result<()>;
}

impl<R> Display for &R where R: Display {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        R::display(*self, f)
    }
}

impl<T> Display for Option<T> where T: Display {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        if let Some(me) = self {
            T::display(me, f)?;
        }
        Ok(())
    }
}

impl<T> Display for Box<T> where T: Display {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        T::display(self, f)
    }
}

impl Display for char {
    fn display(&self, f: &mut impl Writer) -> Result<()> {
        f.write_str(self.encode_utf8(&mut [0u8;4]))
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

