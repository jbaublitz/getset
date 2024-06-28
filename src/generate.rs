use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use proc_macro_error::abort;
use syn::{
    self, ext::IdentExt, spanned::Spanned, Expr, Field, Lit, Meta, MetaNameValue, Visibility,
};

use self::GenMode::*;
use super::parse_attr;

pub struct GenParams {
    pub mode: GenMode,
    pub global_attr: Option<Meta>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum GenMode {
    Get,
    GetCopy,
    Set,
    GetMut,
}

impl GenMode {
    pub fn name(self) -> &'static str {
        match self {
            Get => "get",
            GetCopy => "get_copy",
            Set => "set",
            GetMut => "get_mut",
        }
    }

    pub fn prefix(self) -> &'static str {
        match self {
            Get | GetCopy | GetMut => "",
            Set => "set_",
        }
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Get | GetCopy | Set => "",
            GetMut => "_mut",
        }
    }

    fn is_get(self) -> bool {
        match self {
            GenMode::Get | GenMode::GetCopy | GenMode::GetMut => true,
            GenMode::Set => false,
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

/// Some users want legacy/compatability.
/// (Getters are often prefixed with `get_`)
fn has_prefix_attr(f: &Field, params: &GenParams) -> bool {
    let inner = f
        .attrs
        .iter()
        .filter_map(|v| parse_attr(v, params.mode))
        .filter(|meta| {
            ["get", "get_copy"]
                .iter()
                .any(|ident| meta.path().is_ident(ident))
        })
        .last();

    let wants_prefix = |possible_meta: &Option<Meta>| -> bool {
        match possible_meta {
            Some(Meta::NameValue(meta)) => {
                if let Expr::Lit(expr_lit) = &meta.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        lit_str.value().split(' ').any(|v| v == "with_prefix")
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    };

    wants_prefix(&inner) || wants_prefix(&params.global_attr)
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
        .last()
        .or_else(|| params.global_attr.clone());

    let visibility = parse_visibility(attr.as_ref(), params.mode.name());
    match attr {
        Some(meta) if meta.path().is_ident("skip") => quote! {},
        Some(_) => match params.mode {
            GenMode::Get => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.#field_name
                    }
                }
            }
            GenMode::GetCopy => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.#field_name
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
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> &mut #ty {
                        &mut self.#field_name
                    }
                }
            }
        },
        None => quote! {},
    }
}
