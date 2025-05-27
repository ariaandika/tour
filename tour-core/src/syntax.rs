
/// An expression delimiter.
//
// Opening and closing delimiter must be equal.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// `{{ }}` escaped render.
    Brace,
    /// `{! !}` unescaped render.
    Bang,
    /// `{% %}` escaped render using `std::fmt::Display`.
    Percent,
    /// `{? ?}` escaped render using `std::fmt::Debug`.
    Quest,
    /// `{# #}` unused, same as [`Delimiter::Brace`].
    Hash,
    // /// `{@ @}`
    // At,
    // /// `{$ $}`
    // Dollar,
    // /// `{^ ^}`
    // Caret,
    // /// `{& &}`
    // And,
    // /// `{* *}`
    // Star,
    // /// `{( )}`
    // Paren,
    // /// `{[ ]}`
    // Bracket,
}

impl Delimiter {
    /// Returns [`Some`] if given byte considered as opening delimiter.
    pub fn match_open(ch: u8) -> Option<Self> {
        match ch {
            b'{' => Some(Self::Brace),
            b'!' => Some(Self::Bang),
            b'%' => Some(Self::Percent),
            b'?' => Some(Self::Quest),
            b'#' => Some(Self::Hash),
            _ => None,
        }
    }

    /// Returns [`Some`] if given byte considered as closing delimiter.
    pub fn match_close(ch: u8) -> Option<Self> {
        match ch {
            b'}' => Some(Self::Brace),
            b'!' => Some(Self::Bang),
            b'%' => Some(Self::Percent),
            b'?' => Some(Self::Quest),
            b'#' => Some(Self::Hash),
            _ => None,
        }
    }
}

impl std::fmt::Display for Delimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Brace => f.write_str("brace"),
            Self::Bang => f.write_str("!"),
            Self::Percent => f.write_str("%"),
            Self::Quest => f.write_str("?"),
            Self::Hash => f.write_str("#"),
        }
    }
}

