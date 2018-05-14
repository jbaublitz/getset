use proc_macro2::{Span, Term};
use quote::Tokens;
use std::collections::HashSet;
use syn::{Attribute, Field, Ident, Lit, Meta, MetaNameValue, PathSegment, Type, TypePath};

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

lazy_static! {
    static ref PRIMITIVE_TYPENAMES: HashSet<&'static str> = {
        let mut m = HashSet::new();
        macro_rules! add_types {
                ($m:ident, types: [$( $Type:ident ),*]) => {
                    $(
                        $m.insert(stringify!($Type));
                        )*
                }
        }
        add_types!(m, types: [i8, u8, i16, u16, i32, u32, i64, u64, usize, isize, bool, f32, f64]);
        m
    };
}

fn is_type_primitive(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        qself: ref _qself,
        path,
    }) = ty
    {
        if path.segments.len() == 1 {
            let seg: &PathSegment = path.segments.first().unwrap().value();
            if PRIMITIVE_TYPENAMES.contains(seg.ident.as_ref()) {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub fn implement(field: &Field, mode: GenMode, params: GenParams) -> Tokens {
    let field_name = field
        .clone()
        .ident
        .expect("Expected the field to have a name");
    let type_primitive = is_type_primitive(&field.ty);
    let fn_name = Term::new(
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
                // `#[get = "pub"]` or `#[set = "pub"]`
                Some(Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(ref s),
                    ..
                })) => {
                    let visibility = Term::new(&s.value(), s.span());
                    match mode {
                        GenMode::Get => {
                            if type_primitive {
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
        None => quote!{},
    }
}
