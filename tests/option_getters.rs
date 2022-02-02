#[macro_use]
extern crate getset;

use crate::submodule::other::{
    Generic, Mixed, OptionPath1, OptionPath2, OptionPath3, OptionPath4, Plain, Where,
};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(super::other)`
    pub mod other {

        #[derive(OptionGetters)]
        #[get_option]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: Option<String>,

            /// A doc comment.
            #[get_option = "pub"]
            public_accessible: Option<String>,

            // Prefixed getter.
            #[get_option = "with_prefix"]
            private_prefixed: Option<String>,

            // Prefixed getter.
            #[get_option = "pub with_prefix"]
            public_prefixed: Option<String>,
        }

        impl Default for Plain {
            fn default() -> Plain {
                Plain {
                    private_accessible: Some("17".to_string()),
                    public_accessible: Some("18".to_string()),
                    private_prefixed: Some("19".to_string()),
                    public_prefixed: Some("20".to_string()),
                }
            }
        }

        #[derive(OptionGetters, Default)]
        #[get_option]
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: Option<T>,

            /// A doc comment.
            #[get_option = "pub"]
            public_accessible: Option<T>,
        }

        #[derive(OptionGetters, Getters, Default)]
        #[get_option]
        pub struct Where<T>
        where
            T: Copy + Clone + Default,
        {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: Option<T>,

            /// A doc comment.
            #[get_option = "pub"]
            public_accessible: Option<T>,
        }

        #[derive(Getters, OptionGetters)]
        pub struct Mixed {
            #[getset(get = "pub")]
            field: usize,
            #[getset(get_option = "pub")]
            optional_field: Option<usize>,
        }

        impl Default for Mixed {
            fn default() -> Self {
                Self {
                    field: 101,
                    optional_field: Some(22),
                }
            }
        }

        #[derive(OptionGetters)]
        #[get_option]
        pub struct OptionPath1 {
            /// A doc comment.
            /// Multiple lines, even.
            #[get_option = "pub"]
            public_accessible: std::option::Option<usize>,
        }

        impl Default for OptionPath1 {
            fn default() -> Self {
                Self {
                    public_accessible: Some(42),
                }
            }
        }

        #[derive(OptionGetters)]
        #[get_option]
        pub struct OptionPath2 {
            /// A doc comment.
            /// Multiple lines, even.
            #[get_option = "pub"]
            public_accessible: ::std::option::Option<usize>,
        }

        impl Default for OptionPath2 {
            fn default() -> Self {
                Self {
                    public_accessible: Some(42),
                }
            }
        }

        #[derive(OptionGetters)]
        #[get_option]
        pub struct OptionPath3 {
            /// A doc comment.
            /// Multiple lines, even.
            #[get_option = "pub"]
            public_accessible: core::option::Option<usize>,
        }

        impl Default for OptionPath3 {
            fn default() -> Self {
                Self {
                    public_accessible: Some(42),
                }
            }
        }

        #[derive(OptionGetters)]
        #[get_option]
        pub struct OptionPath4 {
            /// A doc comment.
            /// Multiple lines, even.
            #[get_option = "pub"]
            public_accessible: ::core::option::Option<usize>,
        }

        impl Default for OptionPath4 {
            fn default() -> Self {
                Self {
                    public_accessible: Some(42),
                }
            }
        }

        #[test]
        fn test_plain() {
            let val = Plain::default();
            val.private_accessible();
        }

        #[test]
        fn test_generic() {
            let val = Generic::<usize>::default();
            val.private_accessible();
        }

        #[test]
        fn test_where() {
            let val = Where::<usize>::default();
            val.private_accessible();
        }

        #[test]
        fn test_mixed() {
            let val = Mixed::default();
            val.field();
            val.optional_field();
        }

        #[test]
        fn test_prefixed_plain() {
            let val = Plain::default();
            assert_eq!(Some(&"19".to_string()), val.get_private_prefixed());
        }
    }
}

#[test]
fn test_plain() {
    let val = Plain::default();
    assert_eq!(Some(&"18".to_string()), val.public_accessible());
}

#[test]
fn test_generic() {
    let val = Generic::<usize>::default();
    assert_eq!(None, val.public_accessible());
}

#[test]
fn test_where() {
    let val = Where::<usize>::default();
    assert_eq!(None, val.public_accessible());
}

#[test]
fn test_mixed() {
    let val = Mixed::default();
    assert_eq!(101, *val.field());
    assert_eq!(Some(&22), val.optional_field());
}

#[test]
fn test_prefixed_plain() {
    let val = Plain::default();
    assert_eq!(Some(&"20".to_string()), val.get_public_prefixed());
}

#[test]
fn test_option_path1() {
    let val = OptionPath1::default();
    assert_eq!(Some(&42), val.public_accessible());
}

#[test]
fn test_option_path2() {
    let val = OptionPath2::default();
    assert_eq!(Some(&42), val.public_accessible());
}

#[test]
fn test_option_path3() {
    let val = OptionPath3::default();
    assert_eq!(Some(&42), val.public_accessible());
}

#[test]
fn test_option_path4() {
    let val = OptionPath4::default();
    assert_eq!(Some(&42), val.public_accessible());
}
