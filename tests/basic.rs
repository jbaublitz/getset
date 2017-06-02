#[macro_use]
extern crate getset;

use submodule::other::Foo;

// For testing `pub(super)`
mod submodule {
    // use self::other::Foo;
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(Getters, Setters, Default)]
        pub struct Foo {
            /// A doc comment.
            /// Multiple lines, even.
            #[get]
            private_get: usize,

            /// A doc comment.
            /// Multiple lines, even.
            #[set]
            private_set: usize,

            /// A doc comment.
            #[get = "pub"]
            public_accessible_get: usize,

            /// A doc comment.
            #[set = "pub"]
            public_accessible_set: usize,

            // /// A doc comment.
            // #[get = "pub(crate)"]
            // crate_accessible_get: usize,

            // /// A doc comment.
            // #[set = "pub(crate)"]
            // crate_accessible_set: usize,

            // /// A doc comment.
            // #[get = "pub(super)"]
            // super_accessible_get: usize,

            // /// A doc comment.
            // #[set = "pub(super)"]
            // super_accessible_set: usize,

            // /// A doc comment.
            // #[get = "pub(in super::other)"]
            // scope_accessible_get: usize,

            // /// A doc comment.
            // #[set = "pub(in super::other)"]
            // scope_accessible_set: usize,

            /// A doc comment.
            #[get]
            #[set]
            private_accessible_get_set: usize,

            /// A doc comment.
            #[get = "pub"]
            #[set = "pub"]
            public_accessible_get_set: usize,
            
            // /// A doc comment.
            // #[get = "pub(crate)"]
            // #[set = "pub(crate)"]
            // crate_accessible_get_set: usize,

            // /// A doc comment.
            // #[get = "pub(super)"]
            // #[set = "pub(super)"]
            // super_accessible_get_set: usize,
            
            // /// A doc comment.
            // #[get = "pub(in super::other)"]
            // #[set = "pub(in super::other)"]
            // scope_accessible_get_set: usize,
        }
        
        #[test]
        fn test_private() {
            let mut foo = Foo::default();
            foo.private_get();
            foo.set_private_set(1);
            foo.private_accessible_get_set();
            foo.set_private_accessible_get_set(1);
        }
        
        // #[test]
        // fn test_super_other() {
        //     let mut foo = Foo::default();
        //     foo.scope_accessible_get();
        //     foo.set_scope_accessible_set(1);
        //     foo.scope_accessible_get_set();
        //     foo.set_scope_accessible_get_set(1);
        // }
    }
    // #[test]
    // fn test_super() {
    //     let mut foo = Foo::default();
    //     foo.super_accessible_get();
    //     foo.set_super_accessible_set(1);
    //     foo.super_accessible_get_set();
    //     foo.set_super_accessible_get_set(1);
    // }
}

// #[test]
// fn test_crate() {
//     let mut foo = Foo::default();
//     foo.crate_accessible_get();
//     foo.set_crate_accessible_set(1);
//     foo.crate_accessible_get_set();
//     foo.set_crate_accessible_get_set(1);
// }

#[test]
fn test_public() {
    let mut foo = Foo::default();
    foo.public_accessible_get();
    foo.set_public_accessible_set(1);
    foo.public_accessible_get_set();
    foo.set_public_accessible_get_set(1);
}