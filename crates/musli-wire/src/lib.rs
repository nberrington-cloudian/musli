//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/musli/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)
//!
//! Fully upgrade stable format for [Müsli] suitable for network communication.
//!
//! Wire encoding is fully upgrade stable:
//!
//! * ✔ Can tolerate missing fields if they are annotated with
//!   `#[musli(default)]`.
//! * ✔ Can skip over unknown fields.
//!
//! This means that it's suitable as a wire format, since the data model can
//! evolve independently among clients. Once some clients are upgraded they will
//! start sending unknown fields which non-upgraded clients will be forced to
//! skip over for the duration of the upgrade.
//!
//! ```rust
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Version1 {
//!     name: String,
//! }
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Version2 {
//!     name: String,
//!     #[musli(default)]
//!     age: Option<u32>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let version2 = musli_wire::to_buffer(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(62),
//! })?;
//!
//! let version1: Version1 = musli_wire::decode(version2.as_slice())?;
//!
//! assert_eq!(version1, Version1 {
//!     name: String::from("Aristotle"),
//! });
//! # Ok(()) }
//! ```
//!
//! # Configuring
//!
//! To configure the behavior of the wire format you can use the [Encoding]
//! type:
//!
//! ```rust
//! use musli_wire::Encoding;
//! use musli_wire::int::{Fixed, Variable};
//! use musli::{Encode, Decode};
//! use musli::mode::DefaultMode;
//!
//! const CONFIG: Encoding<DefaultMode, Fixed, Variable, 128> = Encoding::new()
//!     .with_fixed_integers()
//!     .with_max_pack::<128>();
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Struct<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut out = Vec::new();
//!
//! let expected = Struct {
//!     name: "Aristotle",
//!     age: 61,
//! };
//!
//! CONFIG.encode(&mut out, &expected)?;
//! let actual = CONFIG.decode(&out[..])?;
//!
//! assert_eq!(expected, actual);
//! # Ok(()) }
//! ```
//!
//! # Implementation details
//!
//! Each field is prefix *typed* with a single byte tag that allows a receiver
//! to figure out exactly how much should be skipped over.
//!
//! Packed items are prefix-length encoded, and have a limited size. Its exact
//! length is defined by [MAX_INLINE_LEN] and can be modified with
//! [Encoding::with_max_pack].
//!
//! [default encoding format]: https://docs.rs/musli-wire/latest/musli-wire/struct.Encoding.html
//! [MAX_INLINE_LEN]: https://docs.rs/musli-wire/latest/musli_wire/tag/constant.MAX_INLINE_LEN.html
//! [Müsli]: https://docs.rs/musli
//! [Encoding::with_max_pack]: https://docs.rs/musli-wire/latest/musli_wire/encoding/struct.Encoding.html#method.with_max_pack
//! [Encoding]: https://docs.rs/musli-wire/latest/musli-wire/struct.Encoding.html

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod de;
mod en;
pub mod encoding;
mod integer_encoding;
pub mod tag;
#[cfg(feature = "test")]
#[macro_use]
pub mod test;

pub use self::encoding::{decode, encode, from_slice, to_buffer, to_fixed_bytes, Encoding};
#[cfg(feature = "std")]
pub use self::encoding::{to_vec, to_writer};
#[cfg(feature = "test")]
pub use self::test::{transcode, Typed};
#[doc(inline)]
pub use musli_common::*;
