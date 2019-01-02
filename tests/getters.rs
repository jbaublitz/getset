#[macro_use]
extern crate getset;

use submodule::other::{Generic, Plain, Where};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(Getters)]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            #[get]
            private_accessible: usize,

            /// A doc comment
            #[get]
            #[deref]
            private_accessible_deref: u32,

            /// A doc comment.
            #[get = "pub"]
            public_accessible: usize,

            /// A doc comment
            #[get = "pub"]
            #[deref]
            public_accessible_deref: u32,
            // /// A doc comment.
            // #[get = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[get = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[get = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        impl Default for Plain {
            fn default() -> Plain {
                Plain {
                    private_accessible: 17,
                    private_accessible_deref: 18,
                    public_accessible: 19,
                    public_accessible_deref: 20,
                }
            }
        }

        #[derive(Getters, Default)]
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            #[get]
            private_accessible: T,

            /// A doc comment.
            #[get]
            #[deref]
            private_accessible_deref: T,

            /// A doc comment.
            #[get = "pub"]
            public_accessible: T,

            /// A doc comment.
            #[get = "pub"]
            #[deref]
            public_accessible_deref: T,
            // /// A doc comment.
            // #[get = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[get = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[get = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(Getters, Default)]
        pub struct Where<T>
        where
            T: Copy + Clone + Default,
        {
            /// A doc comment.
            /// Multiple lines, even.
            #[get]
            private_accessible: T,

            /// A doc comment.
            #[get]
            #[deref]
            private_accessible_deref: T,

            /// A doc comment.
            #[get = "pub"]
            public_accessible: T,

            /// A doc comment.
            #[get = "pub"]
            #[deref]
            public_accessible_deref: T,
            // /// A doc comment.
            // #[get = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[get = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[get = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[test]
        fn test_plain() {
            let val = Plain::default();
            val.private_accessible();
            val.private_accessible_deref();
        }

        #[test]
        fn test_generic() {
            let val = Generic::<usize>::default();
            val.private_accessible();
            val.private_accessible_deref();
        }

        #[test]
        fn test_where() {
            let val = Where::<usize>::default();
            val.private_accessible();
            val.private_accessible_deref();
        }
    }
}

#[test]
fn test_plain() {
    let val = Plain::default();
    assert_eq!(19, *val.public_accessible());
    assert_eq!(20, val.public_accessible_deref());
}

#[test]
fn test_generic() {
    let val = Generic::<usize>::default();
    assert_eq!(usize::default(), *val.public_accessible());
    assert_eq!(usize::default(), val.public_accessible_deref());
}

#[test]
fn test_where() {
    let val = Where::<usize>::default();
    assert_eq!(usize::default(), *val.public_accessible());
    assert_eq!(usize::default(), val.public_accessible_deref());
}
