#[macro_use]
extern crate getset;

use submodule::other::{Generic, Plain, Where};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(Setters, Default)]
        #[set]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: usize,

            /// A doc comment.
            #[set = "pub"]
            public_accessible: usize,

            /// This field is used for testing chaining.
            #[set = "pub"]
            second_public_accessible: bool,
            // /// A doc comment.
            // #[set = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[set = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[set = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(Setters, Default)]
        #[set]
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: T,

            /// A doc comment.
            #[set = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[set = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[set = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[set = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[derive(Setters, Default)]
        #[set]
        pub struct Where<T>
        where
            T: Copy + Clone + Default,
        {
            /// A doc comment.
            /// Multiple lines, even.
            private_accessible: T,

            /// A doc comment.
            #[set = "pub"]
            public_accessible: T,
            // /// A doc comment.
            // #[set = "pub(crate)"]
            // crate_accessible: usize,

            // /// A doc comment.
            // #[set = "pub(super)"]
            // super_accessible: usize,

            // /// A doc comment.
            // #[set = "pub(in super::other)"]
            // scope_accessible: usize,
        }

        #[test]
        #[allow(unused_assignments)]
        fn test_plain() {
            let _ = Plain::default().set_private_accessible(1usize);
        }

        #[test]
        #[allow(unused_assignments)]
        fn test_generic() {
            let _: Generic<i32> = Generic::default().set_private_accessible(1);
        }

        #[test]
        #[allow(unused_assignments)]
        fn test_where() {
            let _: Where<i32> = Where::default().set_private_accessible(1);
        }
    }
}

#[test]
#[allow(unused_assignments)]
fn test_plain() {
    let mut val = Plain::default();
    val = val.set_public_accessible(1usize);
}

#[test]
#[allow(unused_assignments)]
fn test_generic() {
    let mut val: Generic<usize> = Generic::default();
    val = val.set_public_accessible(1usize);
}

#[test]
#[allow(unused_assignments)]
fn test_where() {
    let mut val: Where<usize> = Where::default();
    val = val.set_public_accessible(1usize);
}

#[test]
#[allow(unused_assignments)]
fn test_chaining() {
    let mut val = Plain::default().set_public_accessible(1usize);
    val = val.set_second_public_accessible(true);
}
