# getset

[![Download](https://img.shields.io/crates/d/getset)](https://crates.io/crates/getset)
[![License](https://img.shields.io/crates/l/getset)](https://github.com/Hoverbear/getset/blob/master/LICENSE)
[![Docs](https://docs.rs/getset/badge.svg)](https://docs.rs/getset/)
[![Coverage Status](https://coveralls.io/repos/github/Hoverbear/getset/badge.svg)](https://coveralls.io/github/Hoverbear/getset)

A Rust procedural macros to generate basic getters and setters for struct fields.

## Quick Start

### What you write

```rust
use getset::{CopyGetters, MutGetters, Setters};

#[derive(Setters, MutGetters, CopyGetters)]
#[derive(Default)]
pub struct Foo<T>
where
    T: Copy + Clone + Default,
{
    #[getset(get_copy, set, get_mut)]
    bar: T,
}
```

### What you get

Use [`cargo-expand`](https://github.com/dtolnay/cargo-expand) to view the macro expansion:

```rust
# pub struct Foo<T>
# where
#     T: Copy + Clone + Default,
# {
#     bar: T,
# }
impl<T> Foo<T>
where
    T: Copy + Clone + Default,
{
    #[inline(always)]
    fn bar(&self) -> T {
        self.bar
    }
}

impl<T> Foo<T>
where
    T: Copy + Clone + Default,
{
    #[inline(always)]
    fn set_bar(&mut self, val: T) -> &mut Self {
        self.bar = val;
        self
    }
}

impl<T> Foo<T>
where
    T: Copy + Clone + Default,
{
    #[inline(always)]
    fn bar_mut(&mut self) -> &mut T {
        &mut self.bar
    }
}
```

## Features

### CopyGetters

Derive a getter that returns a copy of the field value.

```rust
# use getset::CopyGetters;
#
#[derive(CopyGetters)]
pub struct Foo {
    #[getset(get_copy)]
    field: i32,
}

let foo = Foo { field: 42 };
assert_eq!(foo.field(), 42);
```

### Getters

Derive a getter that returns a reference to the field.

```rust
# use getset::Getters;
#
#[derive(Getters)]
pub struct Foo<T> {
    #[getset(get)]
    field: T,
}

let foo = Foo { field: String::from("hello") };
assert_eq!(foo.field(), &String::from("hello"));
```

### MutGetters

Derive a getter that returns a mutable reference to the field.

```rust
# use getset::MutGetters;
#
#[derive(MutGetters)]
pub struct Foo {
    #[getset(get_mut)]
    field: i32,
}

let mut foo = Foo { field: 42 };
*foo.field_mut() = 43;
assert_eq!(foo.field, 43);
```

### Setters

Derive a setter.

```rust
# use getset::Setters;
#
#[derive(Setters)]
pub struct Foo {
    #[getset(set)]
    field: i32,
}

let mut foo = Foo { field: 42 };
foo.set_field(43);
assert_eq!(foo.field, 43);
```

### WithSetters

Derive setters that return `Self` to enable chaining.

```rust
# use getset::WithSetters;
#
#[derive(WithSetters)]
#[derive(Default)]
pub struct Foo {
    #[getset(set_with)]
    field1: i32,
    #[getset(set_with)]
    field2: i32,
}

let foo = Foo::default().with_field1(86).with_field2(87);
assert_eq!(foo.field1, 86);
assert_eq!(foo.field2, 87);
```

### Getter Prefix

Although getters with `get_` do not align with the [RFC-344 convention](https://github.com/rust-lang/rfcs/blob/master/text/0344-conventions-galore.md#gettersetter-apis), they can still be generated using the `with_prefix` feature.

```rust
# use getset::Getters;
#
#[derive(Getters)]
pub struct Foo {
    #[getset(get = "with_prefix")]
    field: bool,
}

let foo = Foo { field: true };
let val = foo.get_field();
```

### Visibility

Customize visibility for generated functions at both the field and struct levels. Supported visibility options include `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, and `pub(self)`.

#### Field-Specific Visibility

```rust
mod submodule {
#   use getset::{Getters, Setters};
#
    #[derive(Getters, Setters)]
    #[derive(Default)]
    pub struct Foo {
        #[getset(get = "pub", set)]
        field: i32,
    }
}

use submodule::Foo;

let foo = Foo::default();
foo.field();          // Public getter
// foo.set_field(10); // Private setter
```

#### Struct-Level Visibility

```rust
mod submodule {
#   use getset::{Getters, Setters};
#
    #[derive(Getters, Setters)]
    #[derive(Default)]
    #[getset(get = "pub", set")]
    pub struct Foo {
        field1: i32,
        field2: i32,
    }
}

use submodule::Foo;

let foo = Foo::default();
foo.field1();          // Public getter
foo.field2();          // Public getter
// foo.set_field1(10); // Private setter
// foo.set_field2(10); // Private setter
```

### Field-Level and Struct-Level Attributes

Attributes can be applied to fields or the entire struct. Field-level attributes override struct-level settings.

```rust
mod submodule {
#   use getset::{Getters};
#
    #[derive(Getters)]
    #[derive(Default)]
    #[getset(get = "pub")]
    pub struct Foo {
        field1: i32,
        #[getset(get)]
        field2: i32,
    }
}

use submodule::Foo;

let foo = Foo::default();
foo.field1();          // Public getter
// foo.field2();       // Private getter
```

### Hidden Field

Fields can skip getter or setter generation with `#[getset(skip)]`.

```rust
# use getset::{CopyGetters, Setters};
#
#[derive(CopyGetters, Setters)]
#[getset(get_copy, set)]
pub struct Foo {
    #[getset(skip)]
    skipped: String,
    field: i32,
}

let foo = Foo { skipped: String::from("hidden"), field: 42 };
// foo.skipped(); // Getter not generated
```

### For Unary Structs

For unary structs (tuple structs with a single field), `get`, `get_mut`, and `set` functions are generated.

```rust
# use getset::{CopyGetters, Getters, MutGetters, Setters};
#
#[derive(Setters, Getters, MutGetters)]
struct UnaryTuple(#[getset(set, get, get_mut)] i32);

let mut tuple = UnaryTuple(42);
assert_eq!(tuple.get(), &42);
assert_eq!(tuple.get_mut(), &mut 42);
tuple.set(43);
assert_eq!(tuple.get(), &43);

#[derive(CopyGetters)]
struct CopyUnaryTuple(#[getset(get_copy)] i32);

let tuple = CopyUnaryTuple(42);
```
