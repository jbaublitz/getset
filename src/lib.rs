/*!
Getset, we're ready to go!

A procedural macro for generating the most basic getters and setters on fields.

Getters are generated as `fn field(&self) -> &type`, while setters are generated
as `fn field(&mut self, val: type)`.

These macros are not intended to be used on fields which require custom logic
inside of their setters and getters. Just write your own in that case!

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

*/

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Attribute, DataStruct, DeriveInput, Field, Ident, Lit, Meta};

mod generate;
use generate::{GenMode, GenParams};

fn attr_name(attr: &Attribute) -> Option<Ident> {
    attr.interpret_meta().map(|v| v.name())
}

/// Some users want legacy/compatability.
/// (Getters are often prefixed with `get_`)
fn has_prefix_attr(f: &Field) -> bool {
    let inner = f
        .attrs
        .iter()
        .filter(|v| attr_name(v).expect("Could not get attribute") == "get")
        .last()
        .and_then(|v| v.parse_meta().ok());
    match inner {
        Some(Meta::NameValue(meta)) => {
            if let Lit::Str(lit) = meta.lit {
                // Naive tokenization to avoid a possible visibility mod named `with_prefix`.
                lit.value()
                    .split(" ")
                    .find(|v| *v == "with_prefix")
                    .is_some()
            } else {
                false
            }
        }
        _ => false,
    }
}

#[proc_macro_derive(Getters, attributes(get, with_prefix))]
pub fn getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).expect("Couldn't parse for getters");

    // Build the impl
    let gen = produce(&ast, |f| {
        let prefix = if has_prefix_attr(f) { "get_" } else { "" };

        generate::implement(
            f,
            GenMode::Get,
            GenParams {
                attribute_name: "get",
                fn_name_prefix: prefix,
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
        let prefix = if has_prefix_attr(f) { "get_" } else { "" };

        generate::implement(
            f,
            GenMode::GetMut,
            GenParams {
                attribute_name: "get_mut",
                fn_name_prefix: prefix,
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

fn produce(ast: &DeriveInput, worker: fn(&Field) -> TokenStream2) -> TokenStream2 {
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
