# getset

[![Build Status](https://travis-ci.org/Hoverbear/getset.svg?branch=master)](https://travis-ci.org/Hoverbear/getset)
[![Build status](https://ci.appveyor.com/api/projects/status/w8v2poyjwsy5d05k?svg=true)](https://ci.appveyor.com/project/Hoverbear/getset)

Getset, we're ready to go!

A procedural macro for generating the most basic getters and setters on fields.

Getters are generated as `fn field(&self) -> &type`, while setters are generated as `fn field(&mut self, val: type)`.

These macros are not intended to be used on fields which require custom logic inside of their setters and getters. Just write your own in that case!

> Yes! It supports nightly with `pub(crate)` etc!

```rust
#[macro_use]
extern crate getset;

#[derive(Getters, Setters, Default)]
pub struct Foo<T> where T: Copy + Clone + Default {
    /// Doc comments are supported!
    /// Multiline, even.
    #[get]
    private_get: T,

    /// Doc comments are supported!
    /// Multiline, even.
    #[set]
    private_set: T,

    /// Doc comments are supported!
    #[get = "pub"]
    public_accessible_get: T,
    
    /// Doc comments are supported!
    #[set = "pub"]
    public_accessible_set: T,

    // /// Doc comments are supported!
    // #[get = "pub(crate)"]
    // crate_accessible_get: T,

    // /// Doc comments are supported!
    // #[set = "pub(crate)"]
    // crate_accessible_set: T,

    // /// Doc comments are supported!
    // #[get = "pub(super)"]
    // super_accessible_get: T,

    // /// Doc comments are supported!
    // #[set = "pub(super)"]
    // super_accessible_set: T,

    // /// Doc comments are supported!
    // #[get = "pub(in some::other::path)"]
    // scope_accessible_get: T,

    // /// Doc comments are supported!
    // #[set = "pub(in some::other::path)"]
    // scope_accessible_set: T,
    
    /// Doc comments are supported!
    #[get]
    #[set]
    private_accessible_get_set: T,
    
    /// Doc comments are supported!
    #[get = "pub"]
    #[set = "pub"]
    public_accessible_get_set: T,
    
    // /// Doc comments are supported!
    // #[get = "pub(crate)"]
    // #[set = "pub(crate)"]
    // crate_accessible_get_set: T,

    // /// Doc comments are supported!
    // #[get = "pub(super)"]
    // #[set = "pub(super)"]
    // super_accessible_get_set: T,
    
    // /// Doc comments are supported!
    // #[get = "pub(in some::other::path)"]
    // #[set = "pub(in some::other::path)"]
    // scope_accessible_get_set: T,
}

fn main() {
    let mut foo = Foo::default();
    foo.private_get();
    foo.set_private_set(1);
}
```