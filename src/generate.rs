use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_error2::abort;
use syn::{
    self, ext::IdentExt, spanned::Spanned, Expr, Field, GenericArgument, Lit, Meta, MetaNameValue,
    PathArguments, Type, Visibility,
};

use self::GenMode::{Get, GetClone, GetCopy, GetMut, Set, SetWith};
use super::parse_attr;

pub struct GenParams {
    pub mode: GenMode,
    pub global_attr: Option<Meta>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum GenMode {
    Get,
    GetClone,
    GetCopy,
    GetMut,
    Set,
    SetWith,
}

impl GenMode {
    pub fn name(self) -> &'static str {
        match self {
            Get => "get",
            GetClone => "get_clone",
            GetCopy => "get_copy",
            GetMut => "get_mut",
            Set => "set",
            SetWith => "set_with",
        }
    }

    pub fn prefix(self) -> &'static str {
        match self {
            Get | GetClone | GetCopy | GetMut => "",
            Set => "set_",
            SetWith => "with_",
        }
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Get | GetClone | GetCopy | Set | SetWith => "",
            GetMut => "_mut",
        }
    }

    fn is_get(self) -> bool {
        match self {
            Get | GetClone | GetCopy | GetMut => true,
            Set | SetWith => false,
        }
    }
}

// Helper function to extract string from Expr
fn expr_to_string(expr: &Expr) -> Option<String> {
    if let Expr::Lit(expr_lit) = expr {
        if let Lit::Str(s) = &expr_lit.lit {
            Some(s.value())
        } else {
            None
        }
    } else {
        None
    }
}

// Helper function to parse visibility
fn parse_vis_str(s: &str, span: proc_macro2::Span) -> Visibility {
    match syn::parse_str(s) {
        Ok(vis) => vis,
        Err(e) => abort!(span, "Invalid visibility found: {}", e),
    }
}

// Helper function to parse attributes
pub struct FieldAttributes {
    pub visibility: Option<Visibility>,
    pub with_prefix: bool,
    pub as_ref: bool,
    pub as_mut: bool,
    pub optional: bool,
    pub into: bool,
    pub is_const: bool,
}

impl Default for FieldAttributes {
    fn default() -> Self {
        FieldAttributes {
            visibility: None,
            with_prefix: false,
            as_ref: false,
            as_mut: false,
            optional: false,
            into: false,
            is_const: false,
        }
    }
}

pub fn parse_attributes(attr: Option<&Meta>) -> FieldAttributes {
    let mut attrs = FieldAttributes::default();

    let meta = match attr {
        Some(m) => m,
        None => return attrs,
    };

    let Meta::NameValue(nv) = meta else {
        return attrs;
    };

    let s = match expr_to_string(&nv.value) {
        Some(s) => s,
        None => return attrs,
    };

    for word in s.split_whitespace() {
        match word {
            "with_prefix" => attrs.with_prefix = true,
            "as_ref" => attrs.as_ref = true,
            "as_mut" => attrs.as_mut = true,
            "optional" => attrs.optional = true,
            "into" => attrs.into = true,
            "const" => attrs.is_const = true,
            _ => {
                // Parse visibility specifiers
                let vis = parse_vis_str(word, nv.value.span());
                if attrs.visibility.is_some() {
                    abort!(
                        nv.value.span(),
                        "Only one visibility specifier is allowed per attribute"
                    );
                }
                attrs.visibility = Some(vis);
            }
        }
    }

    attrs
}

/// Some users want legacy/compatibility.
/// (Getters are often prefixed with `get_`)
fn has_prefix_attr(f: &Field, params: &GenParams) -> bool {
    // helper function to check if meta has `with_prefix` attribute
    let meta_has_prefix = |meta: &Meta| -> bool {
        if let Meta::NameValue(name_value) = meta {
            if let Some(s) = expr_to_string(&name_value.value) {
                return s.split(" ").any(|v| v == "with_prefix");
            }
        }
        false
    };

    let field_attr_has_prefix = f
        .attrs
        .iter()
        .filter_map(|attr| parse_attr(attr, params.mode))
        .find(|meta| {
            meta.path().is_ident("get")
                || meta.path().is_ident("get_clone")
                || meta.path().is_ident("get_copy")
                || meta.path().is_ident("get_mut")
        })
        .as_ref()
        .is_some_and(meta_has_prefix);

    let global_attr_has_prefix = params.global_attr.as_ref().is_some_and(meta_has_prefix);

    field_attr_has_prefix || global_attr_has_prefix
}

// Helper to extract inner type from Option<T>
fn extract_option_inner(ty: &Type) -> Option<Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return Some(inner.clone());
                    }
                }
            }
        }
    }
    None
}

// Helper to generate as_ref/as_mut return type
fn as_ref_type(ty: &Type) -> TokenStream2 {
    if let Some(inner) = extract_option_inner(ty) {
        quote! { Option<&#inner> }
    } else {
        quote! { &#ty }
    }
}

// Helper to generate as_mut return type
fn as_mut_type(ty: &Type) -> TokenStream2 {
    if let Some(inner) = extract_option_inner(ty) {
        quote! { Option<&mut #inner> }
    } else {
        quote! { &mut #ty }
    }
}

pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    let field_name = field
        .ident
        .clone()
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    let fn_name = if !has_prefix_attr(field, params)
        && (params.mode.is_get())
        && params.mode.suffix().is_empty()
        && field_name.to_string().starts_with("r#")
    {
        field_name.clone()
    } else {
        Ident::new(
            &format!(
                "{}{}{}{}",
                if has_prefix_attr(field, params) && (params.mode.is_get()) {
                    "get_"
                } else {
                    ""
                },
                params.mode.prefix(),
                field_name.unraw(),
                params.mode.suffix()
            ),
            Span::call_site(),
        )
    };
    let ty = field.ty.clone();

    let doc = field.attrs.iter().filter(|v| v.meta.path().is_ident("doc"));

    let attr = field
        .attrs
        .iter()
        .filter_map(|v| parse_attr(v, params.mode))
        .next_back()
        .or_else(|| params.global_attr.clone());

    let attrs = parse_attributes(attr.as_ref());
    let visibility = attrs.visibility.unwrap_or_else(|| {
        parse_vis_str("pub(self)", Span::call_site()) // Default to pub(self) which means not using pub at all
    });
    let const_qual = if attrs.is_const {
        quote! { const }
    } else {
        quote! {}
    };

    match attr {
        // Generate nothing for skipped field
        Some(meta) if meta.path().is_ident("skip") => quote! {},
        Some(_) => match params.mode {
            Get | GetClone | GetCopy => {
                // Validate attribute compatibility for getters
                if attrs.as_mut {
                    abort!(
                        field.span(),
                        "`as_mut` attribute is only allowed for MutGetters"
                    );
                }
                if attrs.optional {
                    abort!(
                        field.span(),
                        "`optional` attribute is only allowed for Setters and WithSetters"
                    );
                }
                if attrs.into {
                    abort!(
                        field.span(),
                        "`into` attribute is only allowed for Setters and WithSetters"
                    );
                }

                if attrs.as_ref {
                    let return_ty = as_ref_type(&ty);
                    quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&self) -> #return_ty {
                            self.#field_name.as_ref()
                        }
                    }
                } else {
                    match params.mode {
                        Get => quote! {
                            #(#doc)*
                            #[inline(always)]
                            #visibility #const_qual fn #fn_name(&self) -> &#ty {
                                &self.#field_name
                            }
                        },
                        GetClone => quote! {
                            #(#doc)*
                            #[inline(always)]
                            #visibility #const_qual fn #fn_name(&self) -> #ty {
                                self.#field_name.clone()
                            }
                        },
                        GetCopy => quote! {
                            #(#doc)*
                            #[inline(always)]
                            #visibility #const_qual fn #fn_name(&self) -> #ty {
                                self.#field_name
                            }
                        },
                        _ => unreachable!(),
                    }
                }
            }
            Set | SetWith => {
                // Validate attribute compatibility for setters
                if attrs.as_ref {
                    abort!(
                        field.span(),
                        "`as_ref` attribute is only allowed for Getters"
                    );
                }
                if attrs.as_mut {
                    abort!(
                        field.span(),
                        "`as_mut` attribute is only allowed for MutGetters"
                    );
                }

                let (arg_ty, set_expr) = if attrs.optional {
                    if let Some(inner_ty) = extract_option_inner(&ty) {
                        if attrs.into {
                            (quote! { impl Into<#inner_ty> }, quote! { Some(val.into()) })
                        } else {
                            (quote! { #inner_ty }, quote! { Some(val) })
                        }
                    } else {
                        abort!(
                            ty.span(),
                            "optional attribute requires Option<T> field type"
                        )
                    }
                } else if attrs.into {
                    (quote! { impl Into<#ty> }, quote! { val.into() })
                } else {
                    (quote! { #ty }, quote! { val })
                };

                match params.mode {
                    Set => quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&mut self, val: #arg_ty) -> &mut Self {
                            self.#field_name = #set_expr;
                            self
                        }
                    },
                    SetWith => quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(mut self, val: #arg_ty) -> Self {
                            self.#field_name = #set_expr;
                            self
                        }
                    },
                    _ => unreachable!(),
                }
            }
            GetMut => {
                // Validate attribute compatibility for mutable getters
                if attrs.as_ref {
                    abort!(
                        field.span(),
                        "`as_ref` attribute is only allowed for Getters"
                    );
                }
                if attrs.optional {
                    abort!(
                        field.span(),
                        "`optional` attribute is only allowed for Setters and WithSetters"
                    );
                }
                if attrs.into {
                    abort!(
                        field.span(),
                        "`into` attribute is only allowed for Setters and WithSetters"
                    );
                }

                if attrs.as_mut {
                    let return_ty = as_mut_type(&ty);
                    quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&mut self) -> #return_ty {
                            self.#field_name.as_mut()
                        }
                    }
                } else {
                    quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&mut self) -> &mut #ty {
                            &mut self.#field_name
                        }
                    }
                }
            }
        },
        None => quote! {},
    }
}

pub fn implement_for_unnamed(field: &Field, params: &GenParams) -> TokenStream2 {
    let doc = field.attrs.iter().filter(|v| v.meta.path().is_ident("doc"));
    let attr = field
        .attrs
        .iter()
        .filter_map(|v| parse_attr(v, params.mode))
        .next_back()
        .or_else(|| params.global_attr.clone());
    let ty = field.ty.clone();
    let attrs = parse_attributes(attr.as_ref());
    let visibility = attrs.visibility.unwrap_or_else(|| {
        parse_vis_str("pub(self)", Span::call_site()) // Default to pub(self) which means not using pub at all
    });
    let const_qual = if attrs.is_const {
        quote! { const }
    } else {
        quote! {}
    };

    match attr {
        // Generate nothing for skipped field
        Some(meta) if meta.path().is_ident("skip") => quote! {},
        Some(_) => match params.mode {
            Get | GetClone | GetCopy => {
                // Validate attribute compatibility for getters
                if attrs.as_mut {
                    abort!(
                        field.span(),
                        "`as_mut` attribute is only allowed for MutGetters"
                    );
                }
                if attrs.optional {
                    abort!(
                        field.span(),
                        "`optional` attribute is only allowed for Setters and WithSetters"
                    );
                }
                if attrs.into {
                    abort!(
                        field.span(),
                        "`into` attribute is only allowed for Setters and WithSetters"
                    );
                }

                let fn_name = Ident::new("get", Span::call_site());
                if attrs.as_ref {
                    let return_ty = as_ref_type(&ty);
                    quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&self) -> #return_ty {
                            self.0.as_ref()
                        }
                    }
                } else {
                    match params.mode {
                        Get => quote! {
                            #(#doc)*
                            #[inline(always)]
                            #visibility #const_qual fn #fn_name(&self) -> &#ty {
                                &self.0
                            }
                        },
                        GetClone => quote! {
                            #(#doc)*
                            #[inline(always)]
                            #visibility #const_qual fn #fn_name(&self) -> #ty {
                                self.0.clone()
                            }
                        },
                        GetCopy => quote! {
                            #(#doc)*
                            #[inline(always)]
                            #visibility #const_qual fn #fn_name(&self) -> #ty {
                                self.0
                            }
                        },
                        _ => unreachable!(),
                    }
                }
            }
            Set | SetWith => {
                // Validate attribute compatibility for setters
                if attrs.as_ref {
                    abort!(
                        field.span(),
                        "`as_ref` attribute is only allowed for Getters"
                    );
                }
                if attrs.as_mut {
                    abort!(
                        field.span(),
                        "`as_mut` attribute is only allowed for MutGetters"
                    );
                }

                let (arg_ty, set_expr) = if attrs.optional {
                    if let Some(inner_ty) = extract_option_inner(&ty) {
                        if attrs.into {
                            (quote! { impl Into<#inner_ty> }, quote! { Some(val.into()) })
                        } else {
                            (quote! { #inner_ty }, quote! { Some(val) })
                        }
                    } else {
                        abort!(
                            ty.span(),
                            "optional attribute requires Option<T> field type"
                        )
                    }
                } else if attrs.into {
                    (quote! { impl Into<#ty> }, quote! { val.into() })
                } else {
                    (quote! { #ty }, quote! { val })
                };

                let fn_name = match params.mode {
                    Set => Ident::new("set", Span::call_site()),
                    SetWith => Ident::new("set_with", Span::call_site()),
                    _ => unreachable!(),
                };

                match params.mode {
                    Set => quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&mut self, val: #arg_ty) -> &mut Self {
                            self.0 = #set_expr;
                            self
                        }
                    },
                    SetWith => quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(mut self, val: #arg_ty) -> Self {
                            self.0 = #set_expr;
                            self
                        }
                    },
                    _ => unreachable!(),
                }
            }
            GetMut => {
                // Validate attribute compatibility for mutable getters
                if attrs.as_ref {
                    abort!(
                        field.span(),
                        "`as_ref` attribute is only allowed for Getters"
                    );
                }
                if attrs.optional {
                    abort!(
                        field.span(),
                        "`optional` attribute is only allowed for Setters and WithSetters"
                    );
                }
                if attrs.into {
                    abort!(
                        field.span(),
                        "`into` attribute is only allowed for Setters and WithSetters"
                    );
                }

                let fn_name = Ident::new("get_mut", Span::call_site());
                if attrs.as_mut {
                    let return_ty = as_mut_type(&ty);
                    quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&mut self) -> #return_ty {
                            self.0.as_mut()
                        }
                    }
                } else {
                    quote! {
                        #(#doc)*
                        #[inline(always)]
                        #visibility #const_qual fn #fn_name(&mut self) -> &mut #ty {
                            &mut self.0
                        }
                    }
                }
            }
        },
        None => quote! {},
    }
}
