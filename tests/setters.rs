#[macro_use]
extern crate getset;

use submodule::other::{Plain, Generic, Where};

// For testing `pub(super)`
mod submodule {
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(Setters, Default)]
        pub struct Plain {
            /// A doc comment.
            /// Multiple lines, even.
            #[set]
            private_accessible: usize,
            
            /// A doc comment.
            #[set = "pub"]
            public_accessible: usize,

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
        pub struct Generic<T: Copy + Clone + Default> {
            /// A doc comment.
            /// Multiple lines, even.
            #[set]
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
        pub struct Where<T> where T: Copy + Clone + Default {
            /// A doc comment.
            /// Multiple lines, even.
            #[set]
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
        fn test_plain() {
            let mut val = Plain::default();
            val.set_private_accessible(1);
        }

        #[test]
        fn test_generic() {
            let mut val = Generic::default();
            val.set_private_accessible(1);
        }

        #[test]
        fn test_where() {
            let mut val = Where::default();
            val.set_private_accessible(1);
        }
    }
}

#[test]
fn test_plain() {
    let mut val = Plain::default();
    val.set_public_accessible(1);
}

#[test]
fn test_generic() {
    let mut val = Generic::default();
    val.set_public_accessible(1);
}

#[test]
fn test_where() {
    let mut val = Where::default();
    val.set_public_accessible(1);
}