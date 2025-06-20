//! The [`TemplWrite`] trait
use crate::Result;

/// Analogous to [`std::fmt::Write`] without the [`Formatter`][std::fmt::Formatter].
pub trait TemplWrite {
    /// render a buffer with escapes
    fn write_str(&mut self, value: &str) -> Result<()>;
}

impl<R> TemplWrite for &mut R where R: TemplWrite {
    fn write_str(&mut self, value: &str) -> Result<()> {
        R::write_str(self, value)
    }
}

impl TemplWrite for Vec<u8> {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.extend_from_slice(value.as_bytes());
        Ok(())
    }
}

impl TemplWrite for String {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.push_str(value);
        Ok(())
    }
}

impl TemplWrite for bytes::BytesMut {
    fn write_str(&mut self, value: &str) -> Result<()> {
        bytes::BufMut::put(self, value.as_bytes());
        Ok(())
    }
}

/// Wrap [`TemplWrite`] to escape input.
///
/// escape based on [OWASP recommendation][1]
///
/// [1]: <https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html>
pub struct Escape<W>(pub W);

impl<W> TemplWrite for Escape<W> where W: TemplWrite {
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

/// Wrap [`std::fmt::Write`] to [`TemplWrite`].
pub struct FmtTemplWrite<F>(pub F);

impl<F: std::fmt::Write> TemplWrite for FmtTemplWrite<F> {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.0.write_str(value).map_err(Into::into)
    }
}

deref!(FmtTemplWrite);

/// Wrap [`std::io::Write`] to [`TemplWrite`].
pub struct IoTemplWrite<F>(pub F);

impl<F: std::io::Write> TemplWrite for IoTemplWrite<F> {
    fn write_str(&mut self, value: &str) -> Result<()> {
        self.0.write_all(value.as_bytes()).map_err(Into::into)
    }
}

deref!(IoTemplWrite);

/// Wrap [`TemplWrite`] to [`std::io::Write`].
///
/// Note that it does [utf8 check][str::from_utf8] every write.
pub struct TemplWriteIo<F>(pub F);

impl<F: TemplWrite> std::io::Write for TemplWriteIo<F> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(s) => match self.0.write_str(s) {
                Ok(_) => Ok(s.len()),
                Err(err) => Err(io(err)),
            },
            Err(err) => Err(io(err)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn io<E: Into<Box<dyn std::error::Error + Send + Sync>>>(err: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err)
}

deref!(TemplWriteIo);

/// Wrap [`TemplWrite`] to [`std::fmt::Write`].
pub struct TemplWriteFmt<F>(pub F);

impl<F: TemplWrite> std::fmt::Write for TemplWriteFmt<F> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self.0.write_str(s) {
            Ok(ok) => Ok(ok),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

deref!(TemplWriteFmt);
