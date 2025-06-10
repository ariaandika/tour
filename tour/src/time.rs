use time::format_description::well_known::Rfc2822;

use crate::{TemplDisplay, TemplWrite, write::TemplWriteIo};

pub use time::{
    Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcDateTime, UtcOffset,
    formatting::Formattable,
};

fmt!(Date);
fmt!(OffsetDateTime);
fmt!(PrimitiveDateTime);
fmt!(Time);
fmt!(UtcDateTime);
fmt!(UtcOffset);

fn io<E: Into<Box<dyn std::error::Error + Send + Sync>>>(err: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err)
}

macro_rules! fmt {
    ($ty:ty) => {
        impl TemplDisplay for $ty {
            fn display(&self, f: &mut impl TemplWrite) -> crate::Result<()> {
                match self.format_into(&mut TemplWriteIo(f), &Rfc2822) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(io(err).into()),
                }
            }
        }
    };
}

pub(crate) use fmt;
