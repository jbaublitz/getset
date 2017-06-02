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

pub(crate) fn produce(ast: &DeriveInput, worker: fn(&Field) -> Tokens) -> Tokens {
    let name = &ast.ident;

    // Is it a struct?
    if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {

        let generated = fields.iter().map(worker).collect::<Vec<_>>();

        quote! {
            impl #name {
                #(#generated)*
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        panic!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}