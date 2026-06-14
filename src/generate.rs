use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{
    self, Expr, Field, GenericArgument, Lit, Meta, PathArguments, Type, Visibility, ext::IdentExt,
    spanned::Spanned,
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

// Helper to split attribute string while respecting parentheses
fn split_attr_string(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;

    for c in s.chars() {
        match c {
            '(' => {
                paren_depth += 1;
                current.push(c);
            }
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
                current.push(c);
            }
            ' ' if paren_depth == 0 => {
                if !current.is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
}

// Helper function to parse attributes
#[derive(Default)]
pub struct FieldAttributes {
    pub visibility: Option<Visibility>,
    pub with_prefix: bool,
    pub optional: bool,
    pub into: bool,
    pub is_const: bool,
    pub skip: bool,
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

    // Split while respecting parentheses
    let tokens = split_attr_string(&s);

    let mut found_visibility = false;

    for token in tokens {
        match token.as_str() {
            "with_prefix" => attrs.with_prefix = true,
            "optional" => attrs.optional = true,
            "into" => attrs.into = true,
            "const" => attrs.is_const = true,
            "skip" => attrs.skip = true,
            _ => {
                if !found_visibility {
                    // Parse visibility - might contain spaces in parentheses
                    let vis = parse_vis_str(&token, nv.value.span());
                    attrs.visibility = Some(vis);
                    found_visibility = true;
                } else {
                    abort!(
                        nv.value.span(),
                        "Unexpected token in attribute: '{}'",
                        token
                    );
                }
            }
        }
    }

    // Validate skip is not combined with other attributes
    if attrs.skip
        & (attrs.with_prefix
            || attrs.optional
            || attrs.into
            || attrs.is_const
            || attrs.visibility.is_some())
    {
        abort!(
            nv.value.span(),
            "The 'skip' attribute cannot be combined with any other parameters"
        );
    }

    attrs
}

/// Some users want legacy/compatibility.
/// (Getters are often prefixed with `get_`)
fn has_prefix_attr(f: &Field, params: &GenParams) -> bool {
    // helper function to check if meta has `with_prefix` attribute
    let meta_has_prefix = |meta: &Meta| -> bool {
        if let Meta::NameValue(name_value) = meta
            && let Some(s) = expr_to_string(&name_value.value)
        {
            return s.split(" ").any(|v| v == "with_prefix");
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

pub fn implement(field: &Field, params: &GenParams) -> TokenStream2 {
    let field_name = field
        .ident
        .clone()
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    let attr = field
        .attrs
        .iter()
        .filter_map(|v| parse_attr(v, params.mode))
        .next_back()
        .or_else(|| params.global_attr.clone());

    let attrs = parse_attributes(attr.as_ref());

    if attr.is_none_or(|attr| attr.path().is_ident("skip")) || attrs.skip {
        return quote! {};
    }

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

    let visibility = attrs
        .visibility
        .unwrap_or_else(|| parse_vis_str("pub(self)", Span::call_site()));
    let const_qual = if attrs.is_const {
        quote! { const }
    } else {
        quote! {}
    };

    match params.mode {
        Get | GetClone | GetCopy => {
            // Validate attribute compatibility for getters
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
        Set | SetWith => {
            let (arg_ty, set_expr) = if attrs.optional {
                if let Some(inner_ty) = extract_option_inner(&ty) {
                    if attrs.into {
                        (
                            quote! { impl ::std::convert::Into<#inner_ty> },
                            quote! { Some(val.into()) },
                        )
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
                (
                    quote! { impl ::std::convert::Into<#ty> },
                    quote! { val.into() },
                )
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

            quote! {
                #(#doc)*
                #[inline(always)]
                #visibility #const_qual fn #fn_name(&mut self) -> &mut #ty {
                    &mut self.#field_name
                }
            }
        }
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
    let attrs = parse_attributes(attr.as_ref());

    if attr.is_none() || attrs.skip {
        return quote! {};
    }

    let ty = field.ty.clone();
    let visibility = attrs
        .visibility
        .unwrap_or_else(|| parse_vis_str("pub(self)", Span::call_site()));
    let const_qual = if attrs.is_const {
        quote! { const }
    } else {
        quote! {}
    };

    match params.mode {
        Get | GetClone | GetCopy => {
            // Validate attribute compatibility for getters
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
        Set | SetWith => {
            let (arg_ty, set_expr) = if attrs.optional {
                if let Some(inner_ty) = extract_option_inner(&ty) {
                    if attrs.into {
                        (
                            quote! { impl ::std::convert::Into<#inner_ty> },
                            quote! { Some(val.into()) },
                        )
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
                (
                    quote! { impl ::std::convert::Into<#ty> },
                    quote! { val.into() },
                )
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
            quote! {
                #(#doc)*
                #[inline(always)]
                #visibility #const_qual fn #fn_name(&mut self) -> &mut #ty {
                    &mut self.0
                }
            }
        }
    }
}
