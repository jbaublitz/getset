use syn;
use quote;
use synstructure::{each_field, BindStyle};

pub(crate) fn implement(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    // Is it a struct?
    if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {

        let match_body = each_field(&ast, &BindStyle::Ref.into(), |binding_info| {
            let attrs = &binding_info.field.attrs;
            println!("{:#?}", attrs);
            quote! {
                ()
            }
        });

        quote! {
            impl #name {
                
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
       panic!("#[derive(Setters)] is only defined for structs, not for enums!");
    }
}