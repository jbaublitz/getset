#[macro_use]
extern crate getset;

use submodule::other::{Generic, Plain, Where};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(RefSetters, Default)]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            #[ref_set]
            private_accessible: usize,

            /// A doc comment.
            #[ref_set = "pub"]
            public_accessible: usize,

            /// This field is used for testing chaining.
            #[ref_set = "pub"]
            second_public_accessible: bool,
            // /// A doc comment.
            // #[ref_set = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[ref_set = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[ref_set = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(RefSetters, Default)]
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            #[ref_set]
            private_accessible: T,

            /// A doc comment.
            #[ref_set = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[ref_set = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[ref_set = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[ref_set = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(RefSetters, Default)]
        pub struct Where<T>
        where
            T: Copy + Clone + Default,
        {
            /// A doc comment.
            /// Multiple lines, even.
            #[ref_set]
            private_accessible: T,

            /// A doc comment.
            #[ref_set = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[ref_set = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[ref_set = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[ref_set = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[test]
        fn test_plain() {
            let mut val: Plain = Plain::default();
            val.ref_set_private_accessible(1usize);
        }

        #[test]
        fn test_generic() {
            let mut val: Generic<i32> = Generic::default();
            val.ref_set_private_accessible(1);
        }

        #[test]
        fn test_where() {
            let mut val: Where<i32> = Where::default();
            val.ref_set_private_accessible(1);
        }
    }
}

#[test]
fn test_plain() {
    let mut val = Plain::default();
    val.ref_set_public_accessible(1usize);
}

#[test]
fn test_generic() {
    let mut val: Generic<usize> = Generic::default();
    val.ref_set_public_accessible(1usize);
}

#[test]
fn test_where() {
    let mut val: Where<usize> = Where::default();
    val.ref_set_public_accessible(1usize);
}

#[test]
fn test_chaining() {
    let mut val = Plain::default();
    val.ref_set_public_accessible(1usize)
        .ref_set_second_public_accessible(true);
}
