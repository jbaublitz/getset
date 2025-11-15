#[macro_use]
extern crate getset;

// For testing the prefix attribute
#[derive(Getters, Default)]
pub struct PrefixTest {
    #[getset(get = "pub", prefix = "custom_")]
    field1: usize,

    #[getset(get = "pub", prefix = "mock_")]
    field2: String,

    #[getset(get = "pub")]
    field3: bool,
}

impl PrefixTest {
    fn new() -> Self {
        PrefixTest {
            field1: 42,
            field2: "hello".to_string(),
            field3: true,
        }
    }
}

#[test]
fn test_custom_prefix() {
    let val = PrefixTest::new();
    assert_eq!(42, *val.custom_field1());
    assert_eq!("hello", val.mock_field2());
    assert_eq!(true, *val.field3());
}

// For testing the global prefix attribute
#[derive(Getters, Default)]
#[getset(prefix = "global_")]
pub struct GlobalPrefixTest {
    #[getset(get = "pub")]
    field1: usize,

    #[getset(skip)]
    field2: String,

    #[getset(get = "pub", prefix = "override_")]
    field3: bool,
}

impl GlobalPrefixTest {
    fn new() -> Self {
        GlobalPrefixTest {
            field1: 100,
            field2: "world".to_string(),
            field3: false,
        }
    }

    // Compile time error if field2 was not properly skipped
    fn field2(&self) -> &String {
        &self.field2
    }
}

#[test]
fn test_global_prefix() {
    let val = GlobalPrefixTest::new();
    assert_eq!(100, *val.global_field1());
    // field2 skipped - we can define our own method
    assert_eq!("world", val.field2());
    assert_eq!(false, *val.override_field3());
}
