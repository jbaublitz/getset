#[macro_use]
extern crate getset;

use submodule::other::{Generic, Plain, Where};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(Getters, Replacers)]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            #[get] #[replace]
            private_accessible: usize,

            /// A doc comment.
            #[get = "pub"] #[replace = "pub"]
            public_accessible: usize,
            // /// A doc comment.
            // #[replace = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[replace = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[replace = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        impl Default for Plain {
            fn default() -> Plain {
                Plain {
                    private_accessible: 17,
                    public_accessible: 18,
                }
            }
        }

        #[derive(Getters, Replacers, Default)]
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            #[get] #[replace]
            private_accessible: T,

            /// A doc comment.
            #[get = "pub"] #[replace = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[replace = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[replace = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[replace = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(Getters, Replacers, Default)]
        pub struct Where<T>
        where
            T: Copy + Clone + Default,
        {
            /// A doc comment.
            /// Multiple lines, even.
            #[get] #[replace]
            private_accessible: T,

            /// A doc comment.
            #[get = "pub"] #[replace = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[replace = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[replace = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[replace = "pub(in super::other)"]
            // scope_accessible: usize,
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
    }
}

#[test]
fn test_plain() {
    let mut val = Plain::default();
    assert_eq!(18, val.replace_public_accessible(19));
    assert_eq!(19, *val.public_accessible());
}

#[test]
fn test_generic() {
    let mut val = Generic::<usize>::default();
    assert_eq!(usize::default(), val.replace_public_accessible(1));
    assert_eq!(1, *val.public_accessible());
}

#[test]
fn test_where() {
    let mut val = Where::<usize>::default();
    assert_eq!(usize::default(), val.replace_public_accessible(1));
    assert_eq!(1, *val.public_accessible());
}
