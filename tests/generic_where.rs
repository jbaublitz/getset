#[macro_use]
extern crate etters;

use submodule::other::Foo;

// For testing `pub(super)`
mod submodule {
    use self::other::Foo;
    // For testing `pub(in super::other)`
    pub mod other {
        #[derive(Getters, Setters, Default)]
        pub struct Foo<T> where T: Copy + Clone + Default {
            #[get]
            private_get: T,

            #[set]
            private_set: T,

            #[get = "pub"]
            public_accessible_get: T,
            
            #[set = "pub"]
            public_accessible_set: T,

            #[get = "pub(crate)"]
            crate_accessible_get: T,

            #[set = "pub(crate)"]
            crate_accessible_set: T,

            #[get = "pub(super)"]
            super_accessible_get: T,

            #[set = "pub(super)"]
            super_accessible_set: T,

            #[get = "pub(in super::other)"]
            scope_accessible_get: T,

            #[set = "pub(in super::other)"]
            scope_accessible_set: T,
            
            #[get]
            #[set]
            private_accessible_get_set: T,
            
            #[get = "pub"]
            #[set = "pub"]
            public_accessible_get_set: T,
            
            #[get = "pub(crate)"]
            #[set = "pub(crate)"]
            crate_accessible_get_set: T,

            #[get = "pub(super)"]
            #[set = "pub(super)"]
            super_accessible_get_set: T,
            
            #[get = "pub(in super::other)"]
            #[set = "pub(in super::other)"]
            scope_accessible_get_set: T,
        }
        
        #[test]
        fn test_private() {
            let mut foo = Foo::default();
            foo.private_get();
            foo.set_private_set(1);
            foo.private_accessible_get_set();
            foo.set_private_accessible_get_set(1);
        }
        
        #[test]
        fn test_super_other() {
            let mut foo = Foo::default();
            foo.scope_accessible_get();
            foo.set_scope_accessible_set(1);
            foo.scope_accessible_get_set();
            foo.set_scope_accessible_get_set(1);
        }
    }
    #[test]
    fn test_super() {
        let mut foo = Foo::default();
        foo.super_accessible_get();
        foo.set_super_accessible_set(1);
        foo.super_accessible_get_set();
        foo.set_super_accessible_get_set(1);
    }
}

#[test]
fn test_crate() {
    let mut foo = Foo::default();
    foo.crate_accessible_get();
    foo.set_crate_accessible_set(1);
    foo.crate_accessible_get_set();
    foo.set_crate_accessible_get_set(1);
}

#[test]
fn test_public() {
    let mut foo = Foo::default();
    foo.public_accessible_get();
    foo.set_public_accessible_set(1);
    foo.public_accessible_get_set();
    foo.set_public_accessible_get_set(1);
}