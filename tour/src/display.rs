//! The [`TemplDisplay`] trait.
use crate::{Result, TemplWrite};

pub trait TemplDisplay {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()>;
}

impl<R> TemplDisplay for &R where R: TemplDisplay {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        R::display(*self, f)
    }
}

impl<T> TemplDisplay for Option<T> where T: TemplDisplay {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        if let Some(me) = self {
            T::display(me, f)?;
        }
        Ok(())
    }
}

impl<T> TemplDisplay for Box<T> where T: TemplDisplay {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        T::display(self, f)
    }
}

impl TemplDisplay for char {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        f.write_str(self.encode_utf8(&mut [0u8;4]))
    }
}

impl TemplDisplay for str {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        f.write_str(self)
    }
}

impl TemplDisplay for &str {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        f.write_str(self)
    }
}

impl TemplDisplay for String {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        f.write_str(self)
    }
}

macro_rules! render_int {
    ($t:ty) => {
        impl TemplDisplay for $t {
            fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
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

/// Wrap [`std::fmt::Display`] to [`TemplDisplay`].
pub struct Display<D>(pub D);

impl<D: std::fmt::Display> TemplDisplay for Display<D> {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        use std::fmt::Write as _;
        let mut f = crate::write::TemplWriteFmt(f);
        write!(&mut f, "{}", self.0).map_err(Into::into)
    }
}

