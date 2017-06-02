use syn::{MetaItem, Lit, Field};
use quote::{Ident, Tokens};

const ATTRIBUTE_NAME: &'static str = "set";
const FN_NAME_PREFIX: &'static str = "set_";

pub fn implement(field: &Field) -> Tokens {
    let field_name = field.clone().ident.expect("Expected the field to have a name");
    let fn_name = Ident::from(format!("{}{}", FN_NAME_PREFIX, field_name));
    let ty = field.ty.clone();
    
    let attr = field.attrs.iter()
        .filter(|v| v.name() == ATTRIBUTE_NAME)
        .last();

    let doc = field.attrs.iter()
        .filter(|v| v.name() == "doc")
        .collect::<Vec<_>>();

    match attr {
        Some(attr) => {
            match attr.value {
                // `#[set]`
                MetaItem::Word(_) => {
                    quote! {
                        #(#doc)*
                        fn #fn_name(&mut self, val: #ty) {
                            self.#field_name = val;
                        }
                    }
                },
                // `#[set = "pub"]`
                MetaItem::NameValue(_, Lit::Str(ref s, _)) => {
                    let visibility = Ident::from(s.clone());
                    quote! {
                        #(#doc)*
                        #visibility fn #fn_name(&mut self, val: #ty) {
                            self.#field_name = val;
                        }
                    }
                },
                // This currently doesn't work, but it might in the future.
                /// ---
                // // `#[set(pub)]`
                // MetaItem::List(_, ref vec) => {
                //     let s = vec.iter().last().expect("No item found in attribute list.");
                //     let visibility = match s {
                //         &NestedMetaItem::MetaItem(MetaItem::Word(ref i)) => Ident::new(format!("{}", i)),
                //         &NestedMetaItem::Literal(Lit::Str(ref l, _)) => Ident::from(l.clone()),
                //         _ => panic!("Unexpected attribute parameters."),
                //     };
                //     quote! {
                //         #visibility fn #fn_name(&self) -> &#ty {
                //             &self.#field_name
                //         }
                //     }
                // },
                _ => panic!("Unexpected attribute parameters."),
            }
        },
        // Don't need to do anything.
        None => quote! { }
    }
}