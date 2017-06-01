use syn::{self, MetaItem, Lit};
use quote;
use synstructure::{each_field, BindStyle};

const ATTRIBUTE_NAME: &'static str = "get";

pub(crate) fn implement(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    // Is it a struct?
    if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {

        let match_body = each_field(&ast, &BindStyle::Ref.into(), |binding_info| {
            let field = binding_info.field;
            let field_name = field.clone().ident.expect("Expected the field to have a name");
            let attr = &field.attrs.iter()
                .filter(|v| v.name() == ATTRIBUTE_NAME)
                .last();

            match *attr {
                Some(attr) => {
                    match attr.value {
                        MetaItem::NameValue(ref ident, Lit::Str(ref s, _)) => {
                            let fn_name = quote::Ident::from(format!("get_{}", field_name));
                            let visibility = quote::Ident::from(s.clone());
                            let ty = field.ty.clone();
                            quote! {
                                #visibility fn #fn_name(&self) -> #ty {
                                    self.#field_name
                                }
                            }
                        },
                        _ => quote! { }
                    }
                },
                // Don't need to do anything.
                None => quote! { () }
            }
        });

        quote! {
            impl #name {
                #match_body
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
       panic!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}