/*!
Getset, we're ready to go!

A procedural macro for generating the most basic getters and setters on fields.

Getters are generated as `fn field(&self) -> &type`, while setters are generated as `fn field(&mut self, val: type)`.

These macros are not intended to be used on fields which require custom logic inside of their setters and getters. Just write your own in that case!

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
*/


extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{Field, DeriveInput};
use quote::Tokens;

mod getters;
mod setters;

#[proc_macro_derive(Getters, attributes(get))]
pub fn getters(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    
    // Parse the string representation
    let ast = syn::parse_derive_input(&s).expect("Couldn't parse for getters");

    // Build the impl
    let gen = produce(&ast, getters::implement);

    // Return the generated impl
    gen.parse().unwrap()
}

#[proc_macro_derive(Setters, attributes(set))]
pub fn setters(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    
    // Parse the string representation
    let ast = syn::parse_derive_input(&s).expect("Couldn't parse for setters");

    // Build the impl
    let gen = produce(&ast, setters::implement);
    
    // Return the generated impl
    gen.parse().unwrap()
}

fn produce(ast: &DeriveInput, worker: fn(&Field) -> Tokens) -> Tokens {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {

        let generated = fields.iter().map(worker).collect::<Vec<_>>();

        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #(#generated)*
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        panic!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}