# getset

[![Build Status](https://travis-ci.org/Hoverbear/getset.svg?branch=master)](https://travis-ci.org/Hoverbear/getset)
[![Docs](https://docs.rs/mio/badge.svg)](https://docs.rs/getset/)

Getset, we're ready to go!

A procedural macro for generating the most basic getters and setters on fields.

Getters are generated as `fn field(&self) -> &type`, while setters are generated as `fn field(&mut self, val: type)`.

These macros are not intended to be used on fields which require custom logic inside of their setters and getters. Just write your own in that case!

```rust
#[macro_use]
extern crate getset;

#[derive(Getters, Setters, MutGetters, Default)]
pub struct Foo<T> where T: Copy + Clone + Default {
    /// Doc comments are supported!
    /// Multiline, even.
    #[get] #[set] #[get_mut]
    private: T,

    /// Doc comments are supported!
    /// Multiline, even.
    #[get = "pub"] #[set = "pub"] #[get_mut = "pub"]
    public: T,
}

fn main() {
    let mut foo = Foo::default();
    foo.set_private(1);
    (*foo.private_mut()) += 1;
    assert_eq!(*foo.private(), 2);
}
```

Attributes can be set on struct level for all fields in struct as well. Field level attributes take
precedence.

```rust
#[macro_use]
extern crate getset;

mod submodule {
    #[derive(Getters, Default)]
    #[get = "pub"] // By default add a pub getting for all fields.
    pub struct Foo {
        public: i32,
        #[get] // Override as private
        private: i32,
    }
    fn demo() {
        let mut foo = Foo::default();
        foo.private();
    }
}
fn main() {
    let mut foo = submodule::Foo::default();
    foo.public();
}
```

For some purposes, it's useful to have the `get_` prefix on the getters for
either legacy of compatability reasons. It is done with `get-prefix`.

```rust
#[macro_use]
extern crate getset;

#[derive(Getters, Default)]
pub struct Foo {
    #[get = "pub with_prefix"]
    field: bool,
}

fn main() {
    let mut foo = Foo::default();
    let val = foo.get_field();
}
```