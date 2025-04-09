use fixture::add_1_to_implementation;
use getset::{CloneGetters, CopyGetters};

#[derive(CopyGetters, CloneGetters)]
#[getset(
    get_clone = "with_prefix",
    get_copy,
    impl_attrs = r#"
        #[add_1_to_implementation]
        #[cfg(target_os = "linux")]
        #[add_1_to_implementation]
        #[allow(unused)]
        #[add_1_to_implementation]
    "#
)]
struct Wardrobe {
    shirts: u8,
    pants: u8,
}

#[test]
fn basic() {
    let wardrobe = Wardrobe {
        shirts: 2,
        pants: 1,
    };
    assert_eq!(
        wardrobe.shirts(),
        5,
        "Function attribute not applied correctly."
    );
    assert_eq!(
        wardrobe.pants(),
        4,
        "Function attribute not applied correctly."
    );
    assert_eq!(
        wardrobe.get_shirts(),
        5,
        "Function attribute not applied correctly."
    );
    assert_eq!(
        wardrobe.get_pants(),
        4,
        "Function attribute not applied correctly."
    );
}
