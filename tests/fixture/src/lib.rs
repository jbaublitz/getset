use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned as _, ImplItem, ItemImpl};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn add_1_to_implementation(_attr: TokenStream, item_impl: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item_impl as ItemImpl);
    let attrs = &input.attrs;
    let name = &input.self_ty;
    let (generics, ty_generics, where_clause) = &input.generics.split_for_impl();
    let statements = &input
        .items
        .iter()
        .map(|item| match item {
            ImplItem::Fn(function) => {
                let func_attrs = &function.attrs;
                let sig = &function.sig;
                let body = &function.block.stmts[0];
                quote! {
                    #(#func_attrs)*
                    #sig {
                        #body + 1
                    }
                }
            }
            _ => abort!(item.span(), "Expected a method."),
        })
        .collect::<Vec<_>>();

    quote! {
        #(#attrs)*
        impl #generics #name #ty_generics #where_clause {
            #(#statements)*
        }
    }
    .into()
}
