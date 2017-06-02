# (g|s)etters

A procedural macro for generating the most basic getters and setters on fields.

```rust
#[macro_use]
extern crate etters;

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

    #[get = "pub(in some::other::path)"]
    scope_accessible_get: T,

    #[set = "pub(in some::other::path)"]
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
    
    #[get = "pub(in some::other::path)"]
    #[set = "pub(in some::other::path)"]
    scope_accessible_get_set: T,
}
```