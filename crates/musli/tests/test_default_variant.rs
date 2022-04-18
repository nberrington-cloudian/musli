use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum SeveralVariants {
    Variant1,
    Variant2,
    Variant3,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum OnlyFallback {
    #[musli(tag = 42)]
    Variant4,
    #[musli(default)]
    Fallback,
}

/// Test that enums can use fallback variants when decoding.
#[test]
fn test_fallback_variant() {
    let actual = musli_wire::test::transcode::<_, OnlyFallback>(SeveralVariants::Variant1);
    assert_eq!(actual, OnlyFallback::Fallback);

    let actual = musli_wire::test::transcode::<_, OnlyFallback>(SeveralVariants::Variant2);
    assert_eq!(actual, OnlyFallback::Fallback);

    let actual = musli_wire::test::transcode::<_, OnlyFallback>(SeveralVariants::Variant3);
    assert_eq!(actual, OnlyFallback::Fallback);
}
