use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct TestStruct {
    tag: usize,
}

/// `tag` is used as a variable name to decode the field tag.
/// It should not be confused with an actual field named `tag`.
#[test]
#[cfg(feature = "test")]
fn test_struct_with_field_named_tag() {
    tests::rt!(TestStruct { tag: 42 });
}