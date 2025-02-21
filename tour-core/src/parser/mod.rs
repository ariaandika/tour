//! collection of parsers
//!
//! # `Token` and `TokenTree`
//!
//! `Token` and `TokenTree` is a single unit of syntax, while `TokenTree` may contains a nested
//! token. Both are unit that emitted when tokenizing from a raw source.
//!
//! - token that only difference Punctuation or Identifier
//! - token that have rich variants like literal Plus, Equal, Keyword, etc
//!
//! note that string literal are usually expressed as single token
pub mod token;
