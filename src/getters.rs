use syn;
use quote;

pub(crate) fn implement(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;

    // Is it a struct?
    if let syn::Body::Struct(_) = ast.body {
        quote! {
            impl HelloWorld for #name {
                fn hello_world() {
                    println!("Hello, World! My name is {}", stringify!(#name));
                }
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
       panic!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}