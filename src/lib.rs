/*!
Getset, we're ready to go!

A procedural macro for generating the most basic getters and setters on fields.

Getters are generated as `fn field(&self) -> &type`, while setters are generated as `fn field(&mut self, val: type)`.

These macros are not intended to be used on fields which require custom logic inside of their setters and getters. Just write your own in that case!

```rust
use getset::{Getters, MutGetters, CopyGetters, Setters};

#[derive(Getters, Setters, MutGetters, CopyGetters, Default)]
pub struct Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[get]
    #[set]
    #[get_mut]
    private: T,

    /// Doc comments are supported!
    /// Multiline, even.
    #[get_copy = "pub"]
    #[set = "pub"]
    #[get_mut = "pub"]
    public: T,
}

fn main() {
    let mut foo = Foo::default();
    foo.set_private(1);
    (*foo.private_mut()) += 1;
    assert_eq!(*foo.private(), 2);
}
```

You can use `cargo-expand` to generate the output. Here are the functions that the above generates (Replicate with `cargo expand --example simple`):

```rust,ignore
impl<T> Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[inline(always)]
    fn private(&self) -> &T {
        &self.private
    }
}
impl<T> Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[inline(always)]
    fn set_private(&mut self, val: T) -> &mut Self {
        self.private = val;
        self
    }
    /// Doc comments are supported!
    /// Multiline, even.
    #[inline(always)]
    pub fn set_public(&mut self, val: T) -> &mut Self {
        self.public = val;
        self
    }
}
impl<T> Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[inline(always)]
    fn private_mut(&mut self) -> &mut T {
        &mut self.private
    }
    /// Doc comments are supported!
    /// Multiline, even.
    #[inline(always)]
    pub fn public_mut(&mut self) -> &mut T {
        &mut self.public
    }
}
impl<T> Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[inline(always)]
    pub fn public(&self) -> T {
        self.public
    }
}
```

Attributes can be set on struct level for all fields in struct as well. Field level attributes take
precedence.

```rust
#[macro_use]
extern crate getset;

mod submodule {
    #[derive(Getters, CopyGetters, Default)]
    #[get_copy = "pub"] // By default add a pub getting for all fields.
    pub struct Foo {
        public: i32,
        #[get_copy] // Override as private
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
either legacy of compatability reasons. It is done with `with_prefix`.

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
use syn::{DataStruct, DeriveInput, Meta};
use proc_macro_error::{proc_macro_error, abort_call_site, ResultExt};

mod generate;
use crate::generate::{GenMode, GenParams};

#[proc_macro_derive(Getters, attributes(get, with_prefix))]
#[proc_macro_error]
pub fn getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for getters");
    let params = GenParams {
        attribute_name: "get",
        fn_name_prefix: "",
        fn_name_suffix: "",
        global_attr: parse_global_attr(&ast.attrs, "get"),
    };

    // Build the impl
    let gen = produce(&ast, &GenMode::Get, &params);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(CopyGetters, attributes(get_copy, with_prefix))]
#[proc_macro_error]
pub fn copy_getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for getters");
    let params = GenParams {
        attribute_name: "get_copy",
        fn_name_prefix: "",
        fn_name_suffix: "",
        global_attr: parse_global_attr(&ast.attrs, "get_copy"),
    };

    // Build the impl
    let gen = produce(&ast, &GenMode::GetCopy, &params);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(MutGetters, attributes(get_mut))]
#[proc_macro_error]
pub fn mut_getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for getters");
    let params = GenParams {
        attribute_name: "get_mut",
        fn_name_prefix: "",
        fn_name_suffix: "_mut",
        global_attr: parse_global_attr(&ast.attrs, "get_mut"),
    };

    // Build the impl
    let gen = produce(&ast, &GenMode::GetMut, &params);
    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(Setters, attributes(set))]
#[proc_macro_error]
pub fn setters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for setters");
    let params = GenParams {
        attribute_name: "set",
        fn_name_prefix: "set_",
        fn_name_suffix: "",
        global_attr: parse_global_attr(&ast.attrs, "set"),
    };

    // Build the impl
    let gen = produce(&ast, &GenMode::Set, &params);

    // Return the generated impl
    gen.into()
}

fn parse_global_attr(attrs: &[syn::Attribute], attribute_name: &str) -> Option<Meta> {
    attrs
        .iter()
        .filter_map(|v| {
            let meta = v.parse_meta().expect("attribute");
            if meta.path().is_ident(attribute_name) {
                Some(meta)
            } else {
                None
            }
        })
        .last()
}

fn produce(ast: &DeriveInput, mode: &GenMode, params: &GenParams) -> TokenStream2 {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let generated = fields
            .iter()
            .map(|f| generate::implement(f, mode, params))
            .collect::<Vec<_>>();

        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #(#generated)*
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        abort_call_site!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}
