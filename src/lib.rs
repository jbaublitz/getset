/*!
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
*/

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
#[macro_use]
extern crate lazy_static;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{DataStruct, DeriveInput, Field};

mod generate;
use generate::{GenMode, GenParams};

#[proc_macro_derive(Getters, attributes(get))]
pub fn getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).expect("Couldn't parse for getters");

    // Build the impl
    let gen = produce(&ast, |f| {
        generate::implement(
            f,
            GenMode::Get,
            GenParams {
                attribute_name: "get",
                fn_name_prefix: "",
                fn_name_suffix: "",
            },
        )
    });

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(MutGetters, attributes(get_mut))]
pub fn mut_getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).expect("Couldn't parse for getters");
    // Build the impl
    let gen = produce(&ast, |f| {
        generate::implement(
            f,
            GenMode::GetMut,
            GenParams {
                attribute_name: "get_mut",
                fn_name_prefix: "",
                fn_name_suffix: "_mut",
            },
        )
    });
    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(Setters, attributes(set))]
pub fn setters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).expect("Couldn't parse for setters");

    // Build the impl
    let gen = produce(&ast, |f| {
        generate::implement(
            f,
            GenMode::Set,
            GenParams {
                attribute_name: "set",
                fn_name_prefix: "set_",
                fn_name_suffix: "",
            },
        )
    });

    // Return the generated impl
    gen.into()
}

fn produce(ast: &DeriveInput, worker: fn(&Field) -> Tokens) -> Tokens {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
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
