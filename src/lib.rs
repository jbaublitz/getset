//! Derive Getters and Setters for Structs.
//!
//! This crate provides procedural macros for generating basic getters and setters for struct fields.
//! ## Quick Start
//!
//! ### What you write
//! ```rust
//! use getset::{CopyGetters, MutGetters, Setters};
//!
//! #[derive(Setters, MutGetters, CopyGetters)]
//! #[derive(Default)]
//! pub struct Foo<T>
//! where
//!     T: Copy + Clone + Default,
//! {
//!     #[getset(get_copy, set, get_mut)]
//!     bar: T,
//! }
//! ```
//!
//! ### What you get
//! Use [`cargo-expand`](https://github.com/dtolnay/cargo-expand) to view the macro expansion:
//!
//! ```rust
//! # pub struct Foo<T>
//! # where
//! #     T: Copy + Clone + Default,
//! # {
//! #     bar: T,
//! # }
//! impl<T> Foo<T>
//! where
//!     T: Copy + Clone + Default,
//! {
//!     #[inline(always)]
//!     fn bar(&self) -> T {
//!         self.bar
//!     }
//! }
//! impl<T> Foo<T>
//! where
//!     T: Copy + Clone + Default,
//! {
//!     #[inline(always)]
//!     fn set_bar(&mut self, val: T) -> &mut Self {
//!         self.bar = val;
//!         self
//!     }
//! }
//! impl<T> Foo<T>
//! where
//!     T: Copy + Clone + Default,
//! {
//!     #[inline(always)]
//!     fn bar_mut(&mut self) -> &mut T {
//!         &mut self.bar
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! ### CopyGetters
//!
//! Derive a getter that returns a copy of the field value.
//!
//! ```rust
//! # use getset::CopyGetters;
//! #
//! #[derive(CopyGetters)]
//! pub struct Foo {
//!     #[getset(get_copy)]
//!     field: i32,
//! }
//!
//! let foo = Foo { field: 42 };
//! assert_eq!(foo.field(), 42);
//! ```
//!
//! ### Getters
//!
//! Derive a getter that returns a reference to the field.
//!
//! ```rust
//! # use getset::Getters;
//! #
//! #[derive(Getters)]
//! pub struct Foo<T> {
//!     #[getset(get)]
//!     field: T,
//! }
//!
//! let foo = Foo { field: String::from("hello") };
//! assert_eq!(foo.field(), &String::from("hello"));
//! ```
//!
//! ### MutGetters
//!
//! Derive a getter that returns a mutable reference to the field.
//!
//! ```rust
//! # use getset::MutGetters;
//! #
//! #[derive(MutGetters)]
//! pub struct Foo {
//!     #[getset(get_mut)]
//!     field: i32,
//! }
//!
//! let mut foo = Foo { field: 42 };
//! *foo.field_mut() = 43;
//! assert_eq!(foo.field, 43);
//! ```
//!
//! ### Setters
//!
//! Derive a setter.
//!
//! ```rust
//! # use getset::Setters;
//! #
//! #[derive(Setters)]
//! pub struct Foo {
//!     #[getset(set)]
//!     field: i32,
//! }
//!
//! let mut foo = Foo { field: 42 };
//! foo.set_field(43);
//! assert_eq!(foo.field, 43);
//! ```
//!
//! ### WithSetters
//!
//! Derive setters that returns `Self` to enable chaining.
//!
//! ```rust
//! # use getset::WithSetters;
//! #
//! #[derive(WithSetters)]
//! #[derive(Default)]
//! pub struct Foo {
//!    #[getset(set_with)]
//!    field1: i32,
//!    #[getset(set_with)]
//!    field2: i32,
//! }
//!
//! let foo = Foo::default().with_field1(86).with_field2(87);
//! assert_eq!(foo.field1, 86);
//! assert_eq!(foo.field2, 87);
//! ```
//!
//! ### Getter Prefix
//!
//! Although getters with `get_` does not align with the [RFC-344 convention](https://github.com/rust-lang/rfcs/blob/master/text/0344-conventions-galore.md#gettersetter-apis), they can still be generated using the `with_prefix` feature.
//!
//! ```rust
//! # use getset::Getters;
//! #
//! #[derive(Getters)]
//! pub struct Foo {
//!     #[getset(get = "with_prefix")]
//!     field: bool,
//! }
//!
//! let foo = Foo { field: true };
//! let val = foo.get_field();
//! ```
//!
//! ### Visibility
//!
//! Getset allows customization of visibility for generated functions.
//! You can specify visibility for each field or apply it at the struct level.
//! The visibility values can be any of the supported [Rust visibilities](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
//! Supported visibilities are `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, and `pub(self)`.
//! By default, setters and getters are private.
//!
//! #### Field-Specific Visibility
//!
//! ```rust
//! mod submodule {
//! #   use getset::{Getters, Setters};
//! #
//!     #[derive(Getters, Setters)]
//!     #[derive(Default)]
//!     pub struct Foo {
//!         #[getset(get = "pub", set)]
//!         field: i32,
//!     }
//! }
//!
//! use submodule::Foo;
//!
//! let foo = Foo::default();
//! foo.field();          // Public getter
//! // foo.set_field(10); // Private setter
//! ```
//!
//! #### Struct-Level Visibility
//!
//! ```rust
//! mod submodule {
//! #   use getset::{Getters, Setters};
//! #
//!     #[derive(Getters, Setters)]
//!     #[derive(Default)]
//!     #[getset(get = "pub", set)]
//!     pub struct Foo {
//!         field1: i32,
//!         field2: i32,
//!     }
//! }
//!
//! use submodule::Foo;
//!
//! let foo = Foo::default();
//! foo.field1();          // Public getter
//! foo.field2();          // Public getter
//! // foo.set_field1(10); // Private setter
//! // foo.set_field2(10); // Private setter
//! ```
//! ### Field-Level and Struct-Level Attributes
//!
//! Attributes can be applied to fields or the entire struct. Field-level attributes override struct-level settings.
//!
//! ```rust
//! mod submodule {
//! #   use getset::{Getters};
//! #
//!     #[derive(Getters)]
//!     #[derive(Default)]
//!     #[getset(get = "pub")]
//!     pub struct Foo {
//!         field1: i32,
//!         #[getset(get)]
//!         field2: i32,
//!     }
//! }
//!
//! use submodule::Foo;
//!
//! let foo = Foo::default();
//! foo.field1();          // Public getter
//! // foo.field2();       // Private getter
//! ```
//!
//! ### Hidden Field
//!
//! Fields can skip getter or setter generation with `#[getset(skip)]`.
//!
//! ```rust
//! # use getset::{CopyGetters, Setters};
//! #
//! #[derive(CopyGetters, Setters)]
//! #[getset(get_copy, set)]
//! pub struct Foo {
//!     #[getset(skip)]
//!     skipped: String,
//!     field: i32,
//! }
//!
//! let foo = Foo { skipped: String::from("hidden"), field: 42 };
//! // foo.skipped(); // Getter not generated
//! ```
//!
//! ### For Unary Structs
//!
//! For unary structs (tuple structs with a single field), `get`, `get_mut`, and `set` functions are generated.
//!
//! ```rust
//! # use getset::{CopyGetters, Getters, MutGetters, Setters};
//! #
//! #[derive(Setters, Getters, MutGetters)]
//! struct UnaryTuple(#[getset(set, get, get_mut)] i32);
//!
//! let mut tuple = UnaryTuple(42);
//! assert_eq!(tuple.get(), &42);
//! assert_eq!(tuple.get_mut(), &mut 42);
//! tuple.set(43);
//! assert_eq!(tuple.get(), &43);
//!
//! #[derive(CopyGetters)]
//! struct CopyUnaryTuple(#[getset(get_copy)] i32);
//!
//! let tuple = CopyUnaryTuple(42);
//! ```

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error2::{abort, abort_call_site, proc_macro_error};
use syn::{parse_macro_input, spanned::Spanned, DataStruct, DeriveInput, Meta};

use crate::generate::{GenMode, GenParams};

mod generate;

#[proc_macro_derive(Getters, attributes(get, with_prefix, getset))]
#[proc_macro_error]
pub fn getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = GenParams {
        mode: GenMode::Get,
        global_attr: parse_global_attr(&ast.attrs, GenMode::Get),
    };

    produce(&ast, &params).into()
}

#[proc_macro_derive(CopyGetters, attributes(get_copy, with_prefix, getset))]
#[proc_macro_error]
pub fn copy_getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = GenParams {
        mode: GenMode::GetCopy,
        global_attr: parse_global_attr(&ast.attrs, GenMode::GetCopy),
    };

    produce(&ast, &params).into()
}

#[proc_macro_derive(MutGetters, attributes(get_mut, getset))]
#[proc_macro_error]
pub fn mut_getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = GenParams {
        mode: GenMode::GetMut,
        global_attr: parse_global_attr(&ast.attrs, GenMode::GetMut),
    };

    produce(&ast, &params).into()
}

#[proc_macro_derive(Setters, attributes(set, getset))]
#[proc_macro_error]
pub fn setters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = GenParams {
        mode: GenMode::Set,
        global_attr: parse_global_attr(&ast.attrs, GenMode::Set),
    };

    produce(&ast, &params).into()
}

#[proc_macro_derive(WithSetters, attributes(set_with, getset))]
#[proc_macro_error]
pub fn with_setters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let params = GenParams {
        mode: GenMode::SetWith,
        global_attr: parse_global_attr(&ast.attrs, GenMode::SetWith),
    };

    produce(&ast, &params).into()
}

fn parse_global_attr(attrs: &[syn::Attribute], mode: GenMode) -> Option<Meta> {
    attrs.iter().filter_map(|v| parse_attr(v, mode)).last()
}

fn parse_attr(attr: &syn::Attribute, mode: GenMode) -> Option<syn::Meta> {
    use syn::{punctuated::Punctuated, Token};

    if attr.path().is_ident("getset") {
        let meta_list =
            match attr.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated) {
                Ok(list) => list,
                Err(e) => abort!(attr.span(), "Failed to parse getset attribute: {}", e),
            };

        let (last, skip, mut collected) = meta_list
            .into_iter()
            .inspect(|meta| {
                if !(meta.path().is_ident("get")
                    || meta.path().is_ident("get_copy")
                    || meta.path().is_ident("get_mut")
                    || meta.path().is_ident("set")
                    || meta.path().is_ident("set_with")
                    || meta.path().is_ident("skip"))
                {
                    abort!(meta.path().span(), "unknown setter or getter")
                }
            })
            .fold(
                (None, None, Vec::new()),
                |(last, skip, mut collected), meta| {
                    if meta.path().is_ident(mode.name()) {
                        (Some(meta), skip, collected)
                    } else if meta.path().is_ident("skip") {
                        (last, Some(meta), collected)
                    } else {
                        collected.push(meta);
                        (last, skip, collected)
                    }
                },
            );

        if skip.is_some() {
            // Check if there is any setter or getter used with skip, which is
            // forbidden.
            if last.is_none() && collected.is_empty() {
                skip
            } else {
                abort!(
                    last.or_else(|| collected.pop()).unwrap().path().span(),
                    "use of setters and getters with skip is invalid"
                );
            }
        } else {
            last
        }
    } else if attr.path().is_ident(mode.name()) {
        // If skip is not used, return the last occurrence of matching
        // setter/getter, if there is any.
        attr.meta.clone().into()
    } else {
        None
    }
}

fn produce(ast: &DeriveInput, params: &GenParams) -> TokenStream2 {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        // Handle unary struct
        if matches!(fields, syn::Fields::Unnamed(_)) {
            if fields.len() != 1 {
                abort_call_site!("Only support unary struct!");
            }
            // This unwrap is safe because we know there is exactly one field
            let field = fields.iter().next().unwrap();
            let generated = generate::implement_for_unnamed(field, params);

            quote! {
                impl #impl_generics #name #ty_generics #where_clause {
                    #generated
                }
            }
        } else {
            let generated = fields.iter().map(|f| generate::implement(f, params));

            quote! {
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
