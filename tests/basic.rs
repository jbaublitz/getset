#[macro_use]
extern crate etters;

#[derive(Getters, Setters)]
struct Foo {
    #[get]
    private_get: (),

    #[set]
    private_set: (),

    // #[get = "pub"]
    // public_accessible_get: (),
    
    // #[set = "pub"]
    // public_accessible_set: (),

    // #[get = "pub(crate)"]
    // crate_accessible_get: (),

    // #[set = "pub(crate)"]
    // crate_accessible_set: (),
    
    // #[get = "pub"]
    // #[set = "pub"]
    // public_accessible_get_set: (),
    
    // #[get = "pub(crate)"]
    // #[set = "pub(crate)"]
    // crate_accessible_get_set: (),
}