use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Field, Ident, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

pub struct GenParams {
    pub attribute_name: &'static str,
    pub fn_name_prefix: &'static str,
    pub fn_name_suffix: &'static str,
    pub global_attr: Option<Meta>,
}

#[derive(Clone)]
pub enum GenMode {
    Get,
    Set,
    RefSet,
    GetMut,
}

pub fn attr_tuple(attr: &Attribute) -> Option<(Ident, Meta)> {
    let meta = attr.interpret_meta();
    meta.map(|v| (v.name(), v))
}

pub fn parse_visibility(attr: Option<&Meta>, meta_name: &str) -> Option<Ident> {
    match attr {
        // `#[get = "pub"]` or `#[set = "pub"]`
        Some(Meta::NameValue(MetaNameValue {
            lit: Lit::Str(ref s),
            ident,
            ..
        })) => {
            if ident == meta_name {
                let visibility = Ident::new(&s.value(), Span::call_site());
                Some(visibility)
            } else {
                None
            }
        }
        // `#[get(vis = "pub")]` or `#[set(vis = "pub")]`
        Some(Meta::List(MetaList { nested, .. })) => nested
            .iter()
            .filter_map(|nested_meta| match nested_meta {
                NestedMeta::Meta(meta) => parse_visibility(Some(&meta), "vis"),
                _ => None,
            }).last(),
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
        _ => None,
    }
}

pub fn implement(field: &Field, mode: &GenMode, params: &GenParams) -> TokenStream {
    let field_name = field
        .ident
        .as_ref()
        .expect("Expected the field to have a name");
    let fn_name = Ident::new(
        &format!(
            "{}{}{}",
            params.fn_name_prefix, field_name, params.fn_name_suffix
        ),
        Span::call_site(),
    );
    let ty = &field.ty;

    let mut doc = Vec::new();
    let attr = field
        .attrs
        .iter()
        .filter_map(|v| {
            let tuple = attr_tuple(v).expect("attribute");
            match tuple.0.to_string().as_str() {
                "doc" => {
                    doc.push(v);
                    None
                }
                name if params.attribute_name == name => Some(tuple.1),
                _ => None,
            }
        }).last()
        .or_else(|| params.global_attr.clone());

    let visibility = parse_visibility(attr.as_ref(), params.attribute_name.as_ref());
    match attr {
        Some(_) => match mode {
            GenMode::Get => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.#field_name
                    }
                }
            }
            GenMode::RefSet => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                        self.#field_name = val;
                        self
                    }
                }
            }
            GenMode::Set => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(mut self, val: #ty) -> Self {
                        self.#field_name = val;
                        self
                    }
                }
            }
            GenMode::GetMut => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> &mut #ty {
                        &mut self.#field_name
                    }
                }
            }
        },
        // Don't need to do anything.
        None => quote!{},
    }
}
