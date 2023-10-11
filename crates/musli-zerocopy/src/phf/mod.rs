// Map internals copied from rust-phf under the MIT license.
//
// See:
// https://github.com/rust-phf/rust-phf/tree/b7116ff519415d302c070aa313831cd473b1a911

#[cfg(feature = "alloc")]
pub(crate) mod generator;

mod hashing;

pub use self::map::{Map, MapRef};
mod map;

mod sip;