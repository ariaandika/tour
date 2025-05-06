//! The [`Writer`] trait
use crate::Result;

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

impl Writer for bytes::BytesMut {
    fn write_str(&mut self, value: &str) -> Result<()> {
        bytes::BufMut::put(self, value.as_bytes());
        Ok(())
    }
}

/// Wrap [`Writer`] to escape input.
///
/// escape based on [OWASP recommendation][1]
///
/// [1]: <https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html>
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

