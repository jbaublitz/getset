use attr_name;
use proc_macro2::{Span, Term};
use quote::Tokens;
use syn::{Field, Lit, Meta, MetaNameValue};

const ATTRIBUTE_NAME: &'static str = "get_mut";
const FN_NAME_PREFIX: &'static str = "";
const FN_NAME_SUFFIX: &'static str = "_mut";

pub fn implement(field: &Field) -> Tokens {
    let field_name = field
        .clone()
        .ident
        .expect("Expected the field to have a name");
    let fn_name = Term::new(
        &format!("{}{}{}", FN_NAME_PREFIX, field_name, FN_NAME_SUFFIX),
        Span::call_site(),
    );
    let ty = field.ty.clone();

    let attr = field
        .attrs
        .iter()
        .filter(|v| attr_name(v).expect("attribute name") == ATTRIBUTE_NAME)
        .last();

    let doc = field
        .attrs
        .iter()
        .filter(|v| attr_name(v).expect("attribute name") == "doc")
        .collect::<Vec<_>>();

    match attr {
        Some(attr) => {
            match attr.interpret_meta() {
                // `#[get]`
                Some(Meta::Word(_)) => {
                    quote! {
                        #(#doc)*
                        fn #fn_name(&mut self) -> &mut #ty {
                            &mut self.#field_name
                        }
                    }
                }
                // `#[get = "pub"]`
                Some(Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(ref s),
                    ..
                })) => {
                    let visibility = Term::new(&s.value(), s.span());
                    quote! {
                        #(#doc)*
                        #visibility fn #fn_name(&mut self) -> &mut #ty {
                            &mut self.#field_name
                        }
                    }
                }
                // This currently doesn't work, but it might in the future.
                //
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
