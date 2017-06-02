use syn::{MetaItem, Lit, Field};
use quote::{Ident, Tokens};

const ATTRIBUTE_NAME: &'static str = "get";

pub(crate) fn implement(field: &Field) -> Tokens {
    let field_name = field.clone().ident.expect("Expected the field to have a name");
                
    let attr = field.attrs.iter()
        .filter(|v| v.name() == ATTRIBUTE_NAME)
        .last();

    match attr {
        Some(attr) => {
            match attr.value {
                MetaItem::Word(_) => {
                    let fn_name = Ident::from(format!("{}", field_name));
                    let ty = field.ty.clone();
                    quote! {
                        fn #fn_name(&self) -> #ty {
                            self.#field_name
                        }
                    }
                },
                MetaItem::NameValue(_, Lit::Str(ref s, _)) => {
                    let fn_name = Ident::from(format!("{}", field_name));
                    let visibility = Ident::from(s.clone());
                    let ty = field.ty.clone();
                    quote! {
                        #visibility fn #fn_name(&self) -> #ty {
                            self.#field_name
                        }
                    }
                },
                // Don't need to do anything.
                _ => quote! { }
            }
        },
        // Don't need to do anything.
        None => quote! { }
    }
}