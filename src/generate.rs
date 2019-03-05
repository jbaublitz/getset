use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use syn::{Attribute, Field, Lit, Meta, MetaNameValue};

pub struct GenParams {
    pub attribute_name: &'static str,
    pub fn_name_prefix: &'static str,
    pub fn_name_suffix: &'static str,
}

pub enum GenMode {
    Get,
    Set,
    GetMut,
}

fn attr_name(attr: &Attribute) -> Option<Ident> {
    attr.interpret_meta().map(|v| v.name())
}

pub fn implement(field: &Field, mode: GenMode, params: GenParams) -> TokenStream2 {
    let field_name = field
        .clone()
        .ident
        .expect("Expected the field to have a name");
    let fn_name = Ident::new(
        &format!(
            "{}{}{}",
            params.fn_name_prefix, field_name, params.fn_name_suffix
        ),
        Span::call_site(),
    );
    let ty = field.ty.clone();

    let attr = field
        .attrs
        .iter()
        .filter(|v| attr_name(v).expect("attribute") == params.attribute_name)
        .last();

    let doc = field
        .attrs
        .iter()
        .filter(|v| attr_name(v).expect("attribute") == "doc")
        .collect::<Vec<_>>();

    match attr {
        Some(attr) => {
            match attr.interpret_meta() {
                // `#[get]` or `#[set]`
                Some(Meta::Word(_)) => match mode {
                    GenMode::Get => {
                        quote! {
                            #(#doc)*
                            #[inline(always)]
                            fn #fn_name(&self) -> &#ty {
                                &self.#field_name
                            }
                        }
                    }
                    GenMode::Set => {
                        quote! {
                            #(#doc)*
                            #[inline(always)]
                            fn #fn_name(&mut self, val: #ty) -> &mut Self {
                                self.#field_name = val;
                                self
                            }
                        }
                    }
                    GenMode::GetMut => {
                        quote! {
                            #(#doc)*
                            fn #fn_name(&mut self) -> &mut #ty {
                                &mut self.#field_name
                            }
                        }
                    }
                },
                // `#[get = "pub with_prefix"]` or `#[set = "pub"]`
                Some(Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(ref s),
                    ..
                })) => {
                    let tokens = s.value();
                    let visibility =
                        if let Some(t) = tokens.split(" ").find(|v| v.starts_with("pub")) {
                            Some(Ident::new(&t, s.span()))
                        } else {
                            None
                        };
                    match mode {
                        GenMode::Get => {
                            quote! {
                                #(#doc)*
                                #[inline(always)]
                                #visibility fn #fn_name(&self) -> &#ty {
                                    &self.#field_name
                                }
                            }
                        }
                        GenMode::Set => {
                            quote! {
                                #(#doc)*
                                #[inline(always)]
                                #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                                    self.#field_name = val;
                                    self
                                }
                            }
                        }
                        GenMode::GetMut => {
                            quote! {
                                #(#doc)*
                                #visibility fn #fn_name(&mut self) -> &mut #ty {
                                    &mut self.#field_name
                                }
                            }
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
                //         #visibility fn #fn_name(&self) -> &#ty {
                //             &self.#field_name
                //         }
                //     }
                // },
                _ => panic!("Unexpected attribute parameters."),
            }
        }
        // Don't need to do anything.
        None => quote! {},
    }
}
