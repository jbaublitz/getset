use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{Attribute, Field, Lit, Meta, MetaNameValue};

pub struct GenParams {
    pub attribute_name: &'static str,
    pub optional_attrs: &'static [&'static str],
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
        .expect("expected the field to have a name");
    let fn_name = Ident::new(
        &format!(
            "{}{}{}",
            params.fn_name_prefix, field_name, params.fn_name_suffix
        ),
        Span::call_site(),
    );
    let ty = field.ty.clone();

    let attrs = field
        .attrs
        .iter()
        .filter(|v| {
            let attr_name = attr_name(v).expect("expected attribute");
            attr_name == params.attribute_name
                || params.optional_attrs.iter().any(|n| attr_name == *n)
        })
        .collect::<Vec<_>>();

    let doc = field
        .attrs
        .iter()
        .filter(|v| attr_name(v).expect("expected attribute") == "doc")
        .collect::<Vec<_>>();

    if attrs.is_empty() {
        // Don't need to do anything.
        quote! {}
    } else {
        match mode {
            GenMode::Get => {
                let deref = attrs
                    .iter()
                    .any(|v| attr_name(v).expect("expected attribute") == "deref");
                let vis_attr = attrs
                    .iter()
                    .find(|v| attr_name(v).expect("expected attribute") == params.attribute_name)
                    .expect("no #[get] attribute found");
                match vis_attr.interpret_meta() {
                    // `$[get]`
                    Some(Meta::Word(_)) => {
                        if deref {
                            quote! {
                                #(#doc)*
                                #[inline(always)]
                                fn #fn_name(&self) -> #ty {
                                    self.#field_name
                                }
                            }
                        } else {
                            quote! {
                                #(#doc)*
                                #[inline(always)]
                                fn #fn_name(&self) -> &#ty {
                                    &self.#field_name
                                }
                            }
                        }
                    }
                    // `$[get = "pub"]`
                    Some(Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(ref s),
                        ..
                    })) => {
                        let visibility = Ident::new(&s.value(), s.span());
                        if deref {
                            quote! {
                                #(#doc)*
                                #[inline(always)]
                                #visibility fn #fn_name(&self) -> #ty {
                                    self.#field_name
                                }
                            }
                        } else {
                            quote! {
                                #(#doc)*
                                #[inline(always)]
                                #visibility fn #fn_name(&self) -> &#ty {
                                    &self.#field_name
                                }
                            }
                        }
                    }
                    _ => panic!("unexpected attribute parameters"),
                }
            }
            GenMode::Set => {
                let attr = attrs[0];
                match attr.interpret_meta() {
                    // `$[set]`
                    Some(Meta::Word(_)) => {
                        quote! {
                            #(#doc)*
                            #[inline(always)]
                            fn #fn_name(&mut self, val: #ty) -> &mut Self {
                                self.#field_name = val;
                                self
                            }
                        }
                    }
                    // `$[set = "pub"]`
                    Some(Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(ref s),
                        ..
                    })) => {
                        let visibility = Ident::new(&s.value(), s.span());
                        quote! {
                            #(#doc)*
                            #[inline(always)]
                            #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                                self.#field_name = val;
                                self
                            }
                        }
                    }
                    _ => panic!("unexpected attribute parameters"),
                }
            }
            GenMode::GetMut => {
                let attr = attrs[0];
                match attr.interpret_meta() {
                    // `$[get_mut]`
                    Some(Meta::Word(_)) => {
                        quote! {
                            #(#doc)*
                            fn #fn_name(&mut self) -> &mut #ty {
                                &mut self.#field_name
                            }
                        }
                    }
                    // `$[get_mut = "pub"]`
                    Some(Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(ref s),
                        ..
                    })) => {
                        let visibility = Ident::new(&s.value(), s.span());
                        quote! {
                            #(#doc)*
                            #visibility fn #fn_name(&mut self) -> &mut #ty {
                                &mut self.#field_name
                            }
                        }
                    }
                    _ => panic!("unexpected attribute parameters"),
                    // This currently doesn't work, but it might in the future.
                    // `#[get(pub)]`
                    // MetaItem::List(_, ref vec) => {
                    //     let s = vec.iter().last().expect("No item found in attribute list.");
                    //     let visibility = match s {
                    //         &NestedMetaItem::MetaItem(MetaItem::Word(ref i)) => {
                    //             Ident::new(format!("{}", i))
                    //         }
                    //         &NestedMetaItem::Literal(Lit::Str(ref l, _)) => Ident::from(l.clone()),
                    //         _ => panic!("Unexpected attribute parameters."),
                    //     };
                    //     quote! {
                    //         #visibility fn #fn_name(&self) -> &#ty {
                    //             &self.#field_name
                    //         }
                    //     }
                    // }
                }
            }
        }
    }
}
