//! The [`TemplDisplay`] trait.
use std::fmt;

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

macro_rules! deref {
    ($id:ident) => {
        impl<T> std::ops::Deref for $id<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<F> std::ops::DerefMut for $id<F> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

/// Wrap [`fmt::Display`] to [`TemplDisplay`].
#[derive(Debug)]
pub struct Display<D>(pub D);

impl<D: fmt::Display> TemplDisplay for Display<D> {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        use std::fmt::Write as _;
        let mut f = crate::write::TemplWriteFmt(f);
        write!(&mut f, "{}", self.0).map_err(Into::into)
    }
}

impl<D: fmt::Display> fmt::Display for Display<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

deref!(Display);

/// Wrap [`fmt::Debug`] to [`TemplDisplay`].
pub struct Debug<D>(pub D);

impl<D: fmt::Debug> TemplDisplay for Debug<D> {
    fn display(&self, f: &mut impl TemplWrite) -> Result<()> {
        use std::fmt::Write as _;
        let mut f = crate::write::TemplWriteFmt(f);
        write!(&mut f, "{:?}", self.0).map_err(Into::into)
    }
}

impl<D: fmt::Debug> fmt::Debug for Debug<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

deref!(Debug);
