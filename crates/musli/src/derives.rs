//! # The `Encode` and `Decode` derives
//!
//! The `Encode` and `Decode` derives allows for automatically implementing
//! [Decode] and [Encode].
//!
//! They come with a number of options for customizing their implementation,
//! detailed below.
//!
//! * *Container attributes* are attributes which apply to the `struct` or
//!   `enum`. Like the uses of `#[musli(packed)]` and `#[musli(variant =
//!   "name")]` here:
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(packed)]
//!   struct Struct {
//!       /* the body of the struct */
//!   }
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(variant = "name")]
//!   enum Enum {
//!       /* the body of the struct */
//!   }
//!   ```
//!
//! * *Variant attributes* are attributes which apply to each individual variant
//!   in an `enum`. Like the use of `#[musli(name)]` here:
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(variant = "name")]
//!   enum Enum {
//!       #[musli(tag = "Other")]
//!       Something {
//!           /* variant body */
//!       }
//!   }
//!   ```
//!
//! * *Field attributes* are attributes which apply to each individual field
//!   either in a `struct` or an `enum` variant. Like the uses of
//!   `#[musli(tag)]` here:
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(field = "name")]
//!   struct Struct {
//!       #[musli(tag = "other")]
//!       something: String,
//!   }
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(field = "name")]
//!   enum Enum {
//!       Variant {
//!           #[musli(tag = "other")]
//!           something: String,
//!       }
//!   }
//!   ```
//!
//! ## Container attributes
//!
//! * `#[musli(tag_type = ..)]` indicates which type the `#[musli(tag = ..)]`
//!   attribute on fields or variants should have. Tags can be inferred, but
//!   specifying this field ensures that all tags have a well-defined type.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//!   #[musli(transparent)]
//!   struct CustomTag<'a>(&'a [u8]);
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(tag_type = CustomTag)]
//!   struct Struct {
//!       #[musli(tag = CustomTag(b"name in bytes"))]
//!       name: String,
//!   }
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(tag_type = CustomTag)]
//!   enum EnumWithCustomTag {
//!       #[musli(tag = CustomTag(b"variant one"))]
//!       Variant1 {
//!           /* .. */
//!       },
//!   }
//!   ```
//!
//! * `#[musli(field = "...")]` decides which form of field tag is used for
//!   `#[musli(tagged)]` containers. It can take either `"name"` or `"index"`.
//!   For `"name"` the field name encoded as a string will be used. For
//!   `"index"` its relative index in the struct or tuple
//!
//!   The default value is `#[musli(field = "index")]`.
//!
//! * `#[musli(variant = "...")]` decides which form of variant tag is used for
//!   `#[musli(tagged)]` containers. It can take either `"name"` or `"index"`.
//!   For `"name"` the variant name encoded as a string will be used. For
//!   `"index"` its relative index in the struct or tuple.
//!
//!   The default value is `#[musli(variant = "index")]`.
//!
//! * `#[musli(transparent)]` can only be used on types which have a single
//!   field. It will cause that field to define how that variant is encoded or
//!   decoded transparently without being treated as a field.
//!
//! * `#[musli(packed)]` this attribute will disable all *tagging* and the
//!   structure will simply be encoded with one field following another in the
//!   order in which they are defined.
//!
//!   A caveat of *packed* structures is that they cannot be safely versioned
//!   and the two systems communicating through them need to be using strictly
//!   synchronized representations.
//!
//!   This attribute is useful for performing simple decoding over "raw" bytes.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Struct {
//!     elements: Vec<u32>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let st = Struct {
//!     elements: vec![100, 523],
//! };
//!
//! let mut out = Vec::new();
//! musli_wire::encode(&mut out, &st)?;
//! # Ok(()) }
//! ```
//!
//! ## Variant attributes
//!
//! * `#[musli(tag = ...)]` allows for renaming a variant from its default
//!   tag. Its default tag value is the offset of the variant as its declared
//!   in its container enum.
//!
//! * `#[musli(tag_type = ..)]` indicates which type the `#[musli(tag = ..)]`
//!   attribute on fields in the current variant should have. Tags can be
//!   inferred, but specifying this field ensures that all tags have a
//!   well-defined type.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//!   #[musli(transparent)]
//!   struct CustomTag<'a>(&'a [u8]);
//!
//!   #[derive(Encode, Decode)]
//!   enum Enum {
//!       #[musli(tag_type = CustomTag)]
//!       Variant {
//!           #[musli(tag = CustomTag(b"name in bytes"))]
//!           name: String,
//!       }
//!   }
//!   ```
//!
//! * `#[musli(transparent)]` can only be used on variants which have a single
//!   field. It will cause that field to define how that variant is encoded or
//!   decoded transparently without being treated as a field.
//!
//! * `#[musli(default)]` defines the variant that will be used in case no other
//!   variant matches. Only one such variant can be defined.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//!   enum Animal {
//!       #[musli(tag = "cat")]
//!       Cat,
//!       #[musli(tag = "dog")]
//!       Dog,
//!       #[musli(default)]
//!       Unknown,
//!   }
//!   ```
//!
//! ## Field attributes
//!
//! * `#[musli(tag = ...)]` allows for renaming a field from its default value.
//!   Its default tag value is the offset of the field as its declared in its
//!   container or variant (default or `#[musli(field = "index")]`) or the name
//!   of the field if `#[musli(field = "name")]` is used.
//!
//! * `#[musli(with = <path>)]` specifies the path to a module to use instead of
//!   the fields default [Encode] or [Decode] implementations.
//!
//!   It expects the following functions to be defined, assuming the type of the
//!   field is `Field`.
//!
//!   `encode` for encoding the field, which should match the following
//!   signature:
//!
//!   ```rust,ignore
//!   fn encode<E>(field: &Field, encoder: E) -> Result<(), E::Error>
//!   where
//!      E: Encoder;
//!   ```
//!
//!   `encode` for decoding the field, which should match the following
//!   signature:
//!
//!   ```rust,ignore
//!   fn decode<'de, D>(decoder: D) -> Result<Field, D::Error>
//!   where
//!       D: Decoder<'de>;
//!   ```
//!
//!   ```
//!   # mod types {
//!   use musli::{Encode, Decode};
//!
//!   pub struct CustomUuid(u128);
//!
//!   #[derive(Encode, Decode)]
//!   struct Struct {
//!       #[musli(with = self::custom_uuid)]
//!       name: CustomUuid,
//!   }
//!
//!   mod custom_uuid {
//!       use musli::en::{Encode, Encoder};
//!       use musli::de::{Decode, Decoder};
//!
//!       use super::CustomUuid;
//!
//!       pub fn encode<E>(uuid: &CustomUuid, encoder: E) -> Result<(), E::Error>
//!       where
//!           E: Encoder
//!       {
//!           uuid.0.encode(encoder)
//!       }
//!
//!       pub fn decode<'de, D>(decoder: D) -> Result<CustomUuid, D::Error>
//!       where
//!           D: Decoder<'de>
//!       {
//!           Ok(CustomUuid(u128::decode(decoder)?))
//!       }
//!   }
//!   # }
//!   ```
//!
//! * `#[musli(default)]` constructs the field using [Default::default] in case
//!   it's not available. This is only used when a field is missing during
//!   decoding.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   struct Person {
//!       name: String,
//!       #[musli(default)]
//!       age: Option<u32>,
//!   }
//!   ```
//!
//! * `#[musli(skip_encoding_if = <path>)]` adds a condition to skip encoding a
//!   field entirely if the condition is true. This is very commonly used to
//!   skip over encoding `Option<T>` fields.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   struct Person {
//!       name: String,
//!       #[musli(skip_encoding_if = Option::is_none)]
//!       age: Option<u32>,
//!   }
//!   ```
//!
//! [Encode]: crate::Encode
//! [Decode]: crate::Decode
