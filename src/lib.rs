/*!
Getset, we're ready to go!

A procedural macro for generating the most basic getters and setters on fields.

Getters are generated as `fn field(&self) -> &type`, while setters are generated as `fn field(&mut self, val: type)`.

These macros are not intended to be used on fields which require custom logic inside of their setters and getters. Just write your own in that case!

```rust
use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(Getters, Setters, MutGetters, CopyGetters, Default)]
pub struct Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[getset(get, set, get_mut)]
    private: T,

    /// Doc comments are supported!
    /// Multiline, even.
    #[getset(get_copy = "pub", set = "pub", get_mut = "pub")]
    public: T,
}

let mut foo = Foo::default();
foo.set_private(1);
(*foo.private_mut()) += 1;
assert_eq!(*foo.private(), 2);
```

You can use `cargo-expand` to generate the output. Here are the functions that the above generates (Replicate with `cargo expand --example simple`):

```rust,ignore
use getset::{Getters, MutGetters, CopyGetters, Setters, WithSetters};
pub struct Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[getset(get, get, get_mut)]
    private: T,
    /// Doc comments are supported!
    /// Multiline, even.
    #[getset(get_copy = "pub", set = "pub", get_mut = "pub")]
    public: T,
}
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
mod submodule {
    use getset::{Getters, MutGetters, CopyGetters, Setters, WithSetters};
    #[derive(Getters, CopyGetters, Default)]
    #[getset(get_copy = "pub")] // By default add a pub getting for all fields.
    pub struct Foo {
        public: i32,
        #[getset(get_copy)] // Override as private
        private: i32,
    }
    fn demo() {
        let mut foo = Foo::default();
        foo.private();
    }
}

let mut foo = submodule::Foo::default();
foo.public();
```

For some purposes, it's useful to have the `get_` prefix on the getters for
either legacy of compatibility reasons. It is done with `with_prefix`.

```rust
use getset::{Getters, MutGetters, CopyGetters, Setters, WithSetters};

#[derive(Getters, Default)]
pub struct Foo {
    #[getset(get = "pub with_prefix")]
    field: bool,
}


let mut foo = Foo::default();
let val = foo.get_field();
```

Skipping setters and getters generation for a field when struct level attribute is used
is possible with `#[getset(skip)]`.

```rust
use getset::{CopyGetters, Setters, WithSetters};

#[derive(CopyGetters, Setters, WithSetters)]
#[getset(get_copy, set, set_with)]
pub struct Foo {
    // If the field was not skipped, the compiler would complain about moving
    // a non-copyable type in copy getter.
    #[getset(skip)]
    skipped: String,

    field1: usize,
    field2: usize,
}

impl Foo {
    // It is possible to write getters and setters manually,
    // possibly with a custom logic.
    fn skipped(&self) -> &str {
        &self.skipped
    }

    fn set_skipped(&mut self, val: &str) -> &mut Self {
        self.skipped = val.to_string();
        self
    }

    fn with_skipped(mut self, val: &str) -> Self {
        self.skipped = val.to_string();
        self
    }
}
```

For a unary struct (a tuple struct with a single field),
the macro generates the `get`, `get_mut`, and `set` functions to
provide a getter, a mutable getter, and a setter, respectively.

```rust
use getset::{Getters, MutGetters, CopyGetters, Setters};

#[derive(Setters, Getters, MutGetters)]
struct UnaryTuple(#[getset(set, get, get_mut)] i32);

let mut tup = UnaryTuple(42);
assert_eq!(tup.get(), &42);
assert_eq!(tup.get_mut(), &mut 42);
tup.set(43);
assert_eq!(tup.get(), &43);

#[derive(CopyGetters)]
struct CopyUnaryTuple(#[getset(get_copy)] i32);

let tup = CopyUnaryTuple(42);
```
*/

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error2::{abort, abort_call_site, proc_macro_error};
use syn::{
    parse_macro_input, parse_str, punctuated::Punctuated, spanned::Spanned, Attribute, Data,
    DataStruct, DeriveInput, Fields, ItemImpl, Meta, Token,
};

use crate::generate::{expr_to_string, GenMode, GenParams};

mod generate;

#[proc_macro_derive(Getters, attributes(get, with_prefix, getset))]
#[proc_macro_error]
pub fn getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = make_params(&ast.attrs, GenMode::Get);

    produce(&ast, &params).into()
}

#[proc_macro_derive(CloneGetters, attributes(get_clone, with_prefix, getset))]
#[proc_macro_error]
pub fn clone_getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = make_params(&ast.attrs, GenMode::GetClone);

    produce(&ast, &params).into()
}

#[proc_macro_derive(CopyGetters, attributes(get_copy, with_prefix, getset))]
#[proc_macro_error]
pub fn copy_getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = make_params(&ast.attrs, GenMode::GetCopy);

    produce(&ast, &params).into()
}

#[proc_macro_derive(MutGetters, attributes(get_mut, getset))]
#[proc_macro_error]
pub fn mut_getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = make_params(&ast.attrs, GenMode::GetMut);

    produce(&ast, &params).into()
}

#[proc_macro_derive(Setters, attributes(set, getset))]
#[proc_macro_error]
pub fn setters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = make_params(&ast.attrs, GenMode::Set);

    produce(&ast, &params).into()
}

#[proc_macro_derive(WithSetters, attributes(set_with, getset))]
#[proc_macro_error]
pub fn with_setters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = make_params(&ast.attrs, GenMode::SetWith);

    produce(&ast, &params).into()
}

fn make_params(attrs: &[Attribute], mode: GenMode) -> GenParams {
    let mut impl_attrs = vec![];
    GenParams {
        mode,
        global_attr: attrs
            .iter()
            .filter_map(|v| {
                let (attr, impl_attrs_exist) = parse_attr(v, mode, true);
                if let Some(Meta::NameValue(code)) = &impl_attrs_exist {
                    match expr_to_string(&code.value) {
                        Some(code_str) => {
                            match parse_str::<ItemImpl>(&format!("{} impl _ {{}}", code_str)) {
                                Ok(parsed_impl) => impl_attrs.extend(parsed_impl.attrs),
                                Err(_) => abort!(
                                    code.value.span(),
                                    "Syntax error, expected attributes like #[..]."
                                ),
                            }
                        }
                        None => abort!(code.value.span(), "Expected string."),
                    }
                }
                attr
            })
            .last(),
        impl_attrs,
    }
}

fn parse_attr(
    attr: &Attribute,
    mode: GenMode,
    globally_called: bool,
) -> (Option<Meta>, Option<Meta>) {
    if attr.path().is_ident("getset") {
        let meta_list = match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        {
            Ok(list) => list,
            Err(e) => abort!(attr.span(), "Failed to parse getset attribute: {}", e),
        };

        let (last, skip, impl_attrs, mut collected) = meta_list
            .into_iter()
            .inspect(|meta| {
                if !(meta.path().is_ident("get")
                    || meta.path().is_ident("get_clone")
                    || meta.path().is_ident("get_copy")
                    || meta.path().is_ident("get_mut")
                    || meta.path().is_ident("set")
                    || meta.path().is_ident("set_with")
                    || meta.path().is_ident("skip")
                    || (meta.path().is_ident("impl_attrs") && globally_called))
                {
                    abort!(meta.path().span(), "unknown setter or getter")
                }
            })
            .fold(
                (None, None, None, Vec::new()),
                |(last, skip, impl_attrs, mut collected), meta| {
                    if meta.path().is_ident(mode.name()) {
                        (Some(meta), skip, impl_attrs, collected)
                    } else if meta.path().is_ident("skip") {
                        (last, Some(meta), impl_attrs, collected)
                    } else if meta.path().is_ident("impl_attrs") {
                        (last, skip, Some(meta), collected)
                    } else {
                        collected.push(meta);
                        (last, skip, impl_attrs, collected)
                    }
                },
            );

        if skip.is_some() {
            // Check if there is any setter or getter used with skip, which is
            // forbidden.
            if last.is_none() && collected.is_empty() {
                (skip, impl_attrs)
            } else {
                abort!(
                    last.or_else(|| collected.pop()).unwrap().path().span(),
                    "use of setters and getters with skip is invalid"
                );
            }
        } else {
            (last, impl_attrs)
        }
    } else if attr.path().is_ident(mode.name()) {
        // If skip is not used, return the last occurrence of matching
        // setter/getter, if there is any.
        (attr.meta.clone().into(), None)
    } else {
        (None, None)
    }
}

fn produce(ast: &DeriveInput, params: &GenParams) -> TokenStream2 {
    let impl_attrs = &params.impl_attrs;
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        // Handle unary struct
        if matches!(fields, Fields::Unnamed(_)) {
            if fields.len() != 1 {
                abort_call_site!("Only support unary struct!");
            }
            // This unwrap is safe because we know there is exactly one field
            let field = fields.iter().next().unwrap();
            let generated = generate::implement_for_unnamed(field, params);

            quote! {
                #(#impl_attrs)*
                impl #impl_generics #name #ty_generics #where_clause {
                    #generated
                }
            }
        } else {
            let generated = fields.iter().map(|f| generate::implement(f, params));

            quote! {
                #(#impl_attrs)*
                impl #impl_generics #name #ty_generics #where_clause {
                    #(#generated)*
                }
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        abort_call_site!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}
