use proc_macro_error2::abort;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{
    self, Expr, Field, Lit, Meta, MetaNameValue, Visibility, ext::IdentExt, spanned::Spanned,
};

use self::GenMode::{Get, GetClone, GetCopy, GetMut, Set, SetWith};
use super::parse_attr;

// Helper function to check if a meta contains a specific mode
fn has_mode_in_meta(meta: &Meta, mode: GenMode) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident(mode.name()),
        Meta::List(meta_list) => {
            // Parse the meta list and check if it contains the mode
            if let Ok(nested_metas) = meta_list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) {
                nested_metas
                    .iter()
                    .any(|nested| nested.path().is_ident(mode.name()))
            } else {
                false
            }
        }
        Meta::NameValue(name_value) => name_value.path.is_ident(mode.name()),
    }
}

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

// Helper function to parse visibility attribute
pub fn parse_visibility(attr: Option<&Meta>, meta_name: &str) -> Option<Visibility> {
    let meta = attr?;
    let Meta::NameValue(MetaNameValue { value, path, .. }) = meta else {
        return None;
    };

    if !path.is_ident(meta_name) {
        return None;
    }

    let value_str = expr_to_string(value)?;
    let vis_str = value_str.split(' ').find(|v| *v != "with_prefix")?;

    Some(parse_vis_str(vis_str, value.span()))
}

/// Some users want legacy/compatibility.
/// (Getters are often prefixed with `get_`)
fn get_prefix_attr(f: &Field, params: &GenParams) -> Option<String> {
    // helper function to check if meta has `with_prefix` attribute
    let get_prefix_from_meta = |meta: &Meta| -> Option<String> {
        if let Meta::NameValue(name_value) = meta {
            if meta.path().is_ident("prefix")
                && let Some(s) = expr_to_string(&name_value.value)
            {
                return Some(s);
            } else if let Some(s) = expr_to_string(&name_value.value)
                && s.split(" ").any(|v| v == "with_prefix")
            {
                return Some("get_".to_string());
            }
        }
        None
    };

    let mut prefix = f
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("getset"))
        .find_map(|attr| {
            if let Ok(meta_list) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) {
                meta_list.into_iter().find_map(|meta| {
                    if meta.path().is_ident("prefix") {
                        get_prefix_from_meta(&meta)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .or_else(|| {
            if let Some(meta) = f
                .attrs
                .iter()
                .filter_map(|attr| parse_attr(attr, params.mode))
                .find(|meta| {
                    meta.path().is_ident("get")
                        || meta.path().is_ident("get_clone")
                        || meta.path().is_ident("get_copy")
                        || meta.path().is_ident("get_mut")
                })
            {
                return get_prefix_from_meta(&meta);
            }
            None
        });

    // Prefix can be set globally
    if prefix.is_none()
        && let Some(meta) = params.global_attr.as_ref()
    {
        prefix = get_prefix_from_meta(meta);
    }

    prefix
}

pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    let field_name = field
        .ident
        .clone()
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));
    let prefix = get_prefix_attr(field, params);
    let fn_name = if prefix.is_none()
        && (params.mode.is_get())
        && params.mode.suffix().is_empty()
        && field_name.to_string().starts_with("r#")
    {
        field_name.clone()
    } else {
        Ident::new(
            &format!(
                "{}{}{}{}",
                prefix.unwrap_or("".to_string()),
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
        .or_else(|| {
            // Only inherit global attribute if it contains the current mode
            params.global_attr.as_ref().and_then(|global_attr| {
                if has_mode_in_meta(global_attr, params.mode) {
                    Some(global_attr.clone())
                } else {
                    None
                }
            })
        });

    let visibility = parse_visibility(attr.as_ref(), params.mode.name());
    match attr {
        // Generate nothing for skipped field
        Some(meta) if meta.path().is_ident("skip") => quote! {},
        Some(_) => match params.mode {
            Get => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.#field_name
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
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> &mut #ty {
                        &mut self.#field_name
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
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.0
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
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> &mut #ty {
                        &mut self.0
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
