use quote::{Ident, Tokens};
use syn::{Field, Lit, MetaItem};

const ATTRIBUTE_NAME: &'static str = "get_mut";
const FN_NAME_PREFIX: &'static str = "";
const FN_NAME_SUFFIX: &'static str = "_mut";

pub fn implement(field: &Field) -> Tokens {
    let field_name = field
        .clone()
        .ident
        .expect("Expected the field to have a name");
    let fn_name = Ident::from(format!(
        "{}{}{}",
        FN_NAME_PREFIX, field_name, FN_NAME_SUFFIX
    ));
    let ty = field.ty.clone();

    let attr = field
        .attrs
        .iter()
        .filter(|v| v.name() == ATTRIBUTE_NAME)
        .last();

    let doc = field
        .attrs
        .iter()
        .filter(|v| v.name() == "doc")
        .collect::<Vec<_>>();

    match attr {
        Some(attr) => {
            match attr.value {
                // `#[get]`
                MetaItem::Word(_) => {
                    quote! {
                        #(#doc)*
                        fn #fn_name(&mut self) -> &mut #ty {
                            &mut self.#field_name
                        }
                    }
                }
                // `#[get = "pub"]`
                MetaItem::NameValue(_, Lit::Str(ref s, _)) => {
                    let visibility = Ident::from(s.clone());
                    quote! {
                        #(#doc)*
                        #visibility fn #fn_name(&mut self) -> &mut #ty {
                            &mut self.#field_name
                        }
                    }
                }
                // This currently doesn't work, but it might in the future.
                /// ---
                // // `#[get(pub)]`
                // MetaItem::List(_, ref vec) => {
                //     let s = vec.iter().last().expect("No item found in attribute list.");
                //     let visibility = match s {
                //         &NestedMetaItem::MetaItem(MetaItem::Word(ref i)) => Ident::new(format!("{}", i)),
                //         &NestedMetaItem::Literal(Lit::Str(ref l, _)) => Ident::from(l.clone()),
                //         _ => panic!("Unexpected attribute parameters."),
                //     };
                //     quote! {
                //         #visibility fn #fn_name(&mut self) -> &mut #ty {
                //             &mut self.#field_name
                //         }
                //     }
                // },
                _ => panic!("Unexpected attribute parameters."),
            }
        }
        // Don't need to do anything.
        None => quote!{},
    }
}
