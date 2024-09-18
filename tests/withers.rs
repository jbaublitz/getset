#[macro_use]
extern crate getset;

use crate::submodule::other::{Generic, Plain, Where};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(Withers, Default)]
        #[with]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: usize,

            /// A doc comment.
            #[with = "pub"]
            public_accessible: usize,

            /// This field is used for testing chaining.
            #[with = "pub"]
            second_public_accessible: bool,
            // /// A doc comment.
            // #[with = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[with = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[with = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(Withers, Default)]
        #[with]
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: T,

            /// A doc comment.
            #[with = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[with = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[with = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[with = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(Withers, Default)]
        #[with]
        pub struct Where<T>
        where
            T: Copy + Clone + Default,
        {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: T,

            /// A doc comment.
            #[with = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[with = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[with = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[with = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[test]
        fn test_plain() {
            let mut val = Plain::default();
            val.with_private_accessible(1);
        }

        #[test]
        fn test_generic() {
            let mut val = Generic::default();
            val.with_private_accessible(1);
        }

        #[test]
        fn test_where() {
            let mut val = Where::default();
            val.with_private_accessible(1);
        }
    }
}

#[test]
fn test_plain() {
    let mut val = Plain::default();
    val.with_public_accessible(1);
}

#[test]
fn test_generic() {
    let mut val = Generic::default();
    val.with_public_accessible(1);
}

#[test]
fn test_where() {
    let mut val = Where::default();
    val.with_public_accessible(1);
}

#[test]
fn test_chaining() {
    let mut val = Plain::default();
    val.with_public_accessible(1)
        .with_second_public_accessible(true);
}
