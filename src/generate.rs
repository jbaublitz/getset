use proc_macro_error2::abort;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{
    self, Expr, Field, GenericArgument, Lit, Meta, MetaNameValue, PathArguments, PathSegment, Type,
    TypePath, Visibility, ext::IdentExt, spanned::Spanned,
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

// Helper function to collect named value (named is `meta_name`) from attribute
fn parse_named_value(attr: Option<&Meta>, meta_name: &str) -> Option<String> {
    let meta = attr?;
    let Meta::NameValue(MetaNameValue { path, value, .. }) = meta else {
        return None;
    };

    if !path.is_ident(meta_name) {
        return None;
    }

    let value_str = expr_to_string(value)?;
    Some(value_str)
}

// Helper function to parse visibility
fn parse_vis_str(s: &str, span: proc_macro2::Span) -> Visibility {
    match syn::parse_str(s) {
        Ok(vis) => vis,
        Err(e) => abort!(span, "Invalid visibility found: {}", e),
    }
}

// Helper function to parse visibility attribute
pub fn parse_visibility(attr: Option<&Meta>, meta_name: &str) -> Option<Visibility> {
    let value_str = parse_named_value(attr, meta_name)?;
    let vis_str = value_str.split(' ').find(|v| v.starts_with("pub"))?;

    Some(parse_vis_str(vis_str, attr.span()))
}

fn get_option_inner(seg: &PathSegment, span: proc_macro2::Span) -> &Type {
    if let PathArguments::AngleBracketed(args) = &seg.arguments {
        if let Some(inner_type) = args.args.first() {
            if let GenericArgument::Type(inner_type) = inner_type {
                return inner_type;
            }
        }
    }
    abort!(
        span,
        "as_ref attribute is only supported on `Option` or `Result`"
    )
}

fn get_result_inner(seg: &PathSegment, span: proc_macro2::Span) -> (&Type, &Type) {
    if let PathArguments::AngleBracketed(args) = &seg.arguments {
        let mut args_iter = args.args.iter();
        if let Some(ok_ty) = args_iter.next() {
            if let GenericArgument::Type(ok_ty) = ok_ty {
                if let Some(err_ty) = args_iter.next() {
                    if let GenericArgument::Type(err_ty) = err_ty {
                        return (ok_ty, err_ty);
                    }
                }
            }
        }
    }
    abort!(
        span,
        "as_ref attribute is only supported on `Option` or `Result`"
    )
}

// Helper function to parse as_ref attribute
pub fn parse_as_ref(attr: Option<&Meta>, meta_name: &str) -> bool {
    let Some(value_str) = parse_named_value(attr, meta_name) else {
        return false;
    };
    value_str.split(' ').any(|v| v == "as_ref")
}

fn get_as_ref_return(ty: &Type, span: proc_macro2::Span) -> TokenStream2 {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let Some(last) = path.segments.last() else {
                abort!(
                    span,
                    "as_ref attribute is only supported on `Option` or `Result`"
                );
            };

            if last.ident == "Option" {
                let inner_ty = get_option_inner(last, span);
                return quote! {Option<&#inner_ty>};
            }

            if last.ident == "Result" {
                let (ok_ty, err_ty) = get_result_inner(last, span);
                return quote! {Result<&#ok_ty, &#err_ty>};
            }

            abort!(
                span,
                "as_ref attribute is only supported on `Option` or `Result`"
            )
        }
        _ => abort!(
            span,
            "as_ref attribute is only supported on `Option` or `Result`"
        ),
    }
}

pub fn parse_as_mut(attr: Option<&Meta>, meta_name: &str) -> bool {
    let Some(value_str) = parse_named_value(attr, meta_name) else {
        return false;
    };
    value_str.split(' ').any(|v| v == "as_mut")
}

fn get_as_mut_return(ty: &Type, span: proc_macro2::Span) -> TokenStream2 {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(last) = path.segments.last() {
                if last.ident == "Option" {
                    let inner_ty = get_option_inner(last, span);
                    return quote! {Option<&mut #inner_ty>};
                }

                if last.ident == "Result" {
                    let (ok_ty, err_ty) = get_result_inner(last, span);
                    return quote! {Result<&mut #ok_ty, &mut #err_ty>};
                }
            }

            abort!(
                span,
                "as_mut attribute is only supported on `Option` or `Result`"
            )
        }
        _ => abort!(
            span,
            "as_mut attribute is only supported on `Option` or `Results`"
        ),
    }
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

    let visibility = parse_visibility(attr.as_ref(), params.mode.name());
    match attr {
        // Generate nothing for skipped field
        Some(meta) if meta.path().is_ident("skip") => quote! {},
        Some(_) => match params.mode {
            Get => {
                let (return_code, return_ty) = if parse_as_ref(attr.as_ref(), params.mode.name()) {
                    (
                        quote! {self.#field_name.as_ref()},
                        get_as_ref_return(&ty, field.span()),
                    )
                } else {
                    (quote! {&self.#field_name}, quote! {&#ty})
                };
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #return_ty {
                        #return_code
                    }
                }
            }
            GetClone => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.#field_name.clone()
                    }
                }
            }
            GetCopy => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.#field_name
                    }
                }
            }
            Set => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                        self.#field_name = val;
                        self
                    }
                }
            }
            GetMut => {
                let (return_code, return_ty) = if parse_as_mut(attr.as_ref(), params.mode.name()) {
                    (
                        quote! {self.#field_name.as_mut()},
                        get_as_mut_return(&ty, field.span()),
                    )
                } else {
                    (quote! {&mut self.#field_name}, quote! {&mut #ty})
                };
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> #return_ty {
                        #return_code
                    }
                }
            }
            SetWith => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(mut self, val: #ty) -> Self {
                        self.#field_name = val;
                        self
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
    let visibility = parse_visibility(attr.as_ref(), params.mode.name());

    match attr {
        // Generate nothing for skipped field
        Some(meta) if meta.path().is_ident("skip") => quote! {},
        Some(_) => match params.mode {
            Get => {
                let fn_name = Ident::new("get", Span::call_site());
                let (return_code, return_ty) = if parse_as_ref(attr.as_ref(), params.mode.name()) {
                    (
                        quote! {self.0.as_ref()},
                        get_as_ref_return(&ty, field.span()),
                    )
                } else {
                    (quote! {&self.0}, quote! {&#ty})
                };
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #return_ty {
                        #return_code
                    }
                }
            }
            GetClone => {
                let fn_name = Ident::new("get", Span::call_site());
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.0.clone()
                    }
                }
            }
            GetCopy => {
                let fn_name = Ident::new("get", Span::call_site());
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.0
                    }
                }
            }
            Set => {
                let fn_name = Ident::new("set", Span::call_site());
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                        self.0 = val;
                        self
                    }
                }
            }
            GetMut => {
                let fn_name = Ident::new("get_mut", Span::call_site());
                let (return_code, return_ty) = if parse_as_mut(attr.as_ref(), params.mode.name()) {
                    (
                        quote! {self.0.as_mut()},
                        get_as_mut_return(&ty, field.span()),
                    )
                } else {
                    (quote! {&mut self.0}, quote! {&mut #ty})
                };
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> #return_ty {
                        #return_code
                    }
                }
            }
            SetWith => {
                let fn_name = Ident::new("set_with", Span::call_site());
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(mut self, val: #ty) -> Self {
                        self.0 = val;
                        self
                    }
                }
            }
        },
        None => quote! {},
    }
}
