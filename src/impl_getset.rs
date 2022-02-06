use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, abort_call_site, OptionExt, ResultExt};

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[allow(unused_imports)]
use syn::{
    self, ext::IdentExt, spanned::Spanned, Field, GenericArgument, Lit, Meta, MetaNameValue, Path,
    PathArguments, PathSegment, Visibility,
};
use syn::{DeriveInput, DataStruct};

pub const LEGACY_GETTER_PREFIX: &str = "get_";

#[derive(Clone)]
pub struct GetSetAttr {
    mode: GetSetMode,
    prefix: Option<&'static str>,
}

#[derive(Copy, Clone)]
pub enum GetSetMode {
    Get,
    GetCopy,
    GetOption,
    GetMut,
    Set,
    SetOption,
    Skip,
}

impl GetSetMode {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::GetCopy => "get_copy",
            Self::GetOption => "get_option",
            Self::GetMut => "get_mut",
            Self::Set => "set",
            Self::SetOption => "set_option",
            Self::Skip => "skip",
        }
    }
}

impl PartialEq for GetSetAttr {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}

impl Eq for GetSetAttr {}

impl Hash for GetSetAttr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier().hash(state)
    }
}

impl GetSetAttr {
    pub fn new(meta: &Meta) -> Self {
        // Parse name of atrribute.
        let name = meta
            .path()
            .get_ident()
            .expect_or_abort("Couldn't parse GetSet attributes")
            .to_string();
        // Try to convert attribute name to getter/setter/skip type.
        let mode: GetSetMode = match name.parse() {
            Ok(mode) => mode,
            Err(_) => abort!(&meta, "Not a valid getter/setter mode"),
        };
        // Check if a getter attribute wants a get_ prefix.
        let prefix = match attr_wants_prefix(meta) {
            true => Some(LEGACY_GETTER_PREFIX),
            false => None,
        };
        Self { mode, prefix }
    }

    pub fn skip() -> Self {
        Self {
            mode: GetSetMode::Skip,
            prefix: None,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        self.mode.name()
    }

    pub fn prefix(&self) -> &'static str {
        match (self.mode, self.prefix) {
            (GetSetMode::Set, _) => "set_",
            (GetSetMode::SetOption, _) => "set_",
            (_, Some(prefix)) => prefix,
            _ => "",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self.mode {
            GetSetMode::GetMut => "_mut",
            _ => "",
        }
    }

    pub fn is_get(&self) -> bool {
        !matches!(
            self.mode,
            GetSetMode::Set | GetSetMode::SetOption | GetSetMode::Skip
        )
    }

    pub fn is_option(&self) -> bool {
        matches!(self.mode, GetSetMode::GetOption | GetSetMode::SetOption)
    }

    pub fn identifier(&self) -> &'static str {
        if let Some(prefix) = self.prefix {
            return prefix;
        }
        match self.mode {
            GetSetMode::Get | GetSetMode::GetCopy | GetSetMode::GetOption => "get",
            GetSetMode::GetMut => "get_mut",
            GetSetMode::Set | GetSetMode::SetOption => "set",
            GetSetMode::Skip => "skip",
        }
    }
}

impl std::str::FromStr for GetSetMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Self::Get),
            "get_copy" => Ok(Self::GetCopy),
            "get_option" => Ok(Self::GetOption),
            "get_mut" => Ok(Self::GetMut),
            "set" => Ok(Self::Set),
            "set_option" => Ok(Self::SetOption),
            "skip" => Ok(Self::Skip),
            _ => Err(format!("'{}' is not a valid value for GetSetMode", s)),
        }
    }
}

pub fn parse_visibility(attr: &Meta) -> Option<Visibility> {
    // `#[get = "pub"]` or `#[set = "pub"]`
    if let Meta::NameValue(MetaNameValue {
                               lit: Lit::Str(ref s),
                               ..
                           }) = attr
    {
        s.value().split(' ').find(|v| *v != "with_prefix").map(|v| {
            syn::parse_str(v)
                .map_err(|e| syn::Error::new(s.span(), e))
                .expect_or_abort("invalid visibility found")
        })
    } else {
        None
    }
}

// Check if the attr includes `with_prefix`
pub fn attr_wants_prefix(meta: &Meta) -> bool {
    if let Meta::NameValue(MetaNameValue {
                               lit: Lit::Str(ref lit_str),
                               ..
                           }) = meta
    {
        // Naive tokenization to avoid a possible visibility mod named `with_prefix`.
        lit_str.value().split(' ').any(|v| v == "with_prefix")
    } else {
        false
    }
}

/// Extract the inner type T of an Option<T>. This function is based on the SO answer
/// of David Bernard: https://stackoverflow.com/a/56264023/17134768
fn extract_type_from_option(ty: &syn::Type) -> syn::Type {
    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(extract_option_segment)
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                ref params => abort!(params, "Only one angle-bracketed param is supported"),
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            ref arg => abort!(arg, "Inner type T of Option<T> could not be extracted"),
        })
        .expect_or_abort(&format!(
            "expected Option<T> because of get_option attribute, found {}",
            quote!(#ty)
        ))
        .to_owned()
}

pub fn collect_attr(attrs: &[syn::Attribute]) -> HashMap<GetSetAttr, Meta> {
    use syn::{punctuated::Punctuated, Token};

    let metas: Vec<Meta> = attrs
        .iter()
        .map(|attr| {
            if attr.path.is_ident("getset") {
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .unwrap_or_abort()
                    .into_iter()
                    .collect::<Vec<_>>()
            } else {
                match attr.parse_meta() {
                    Ok(meta) => vec![meta],
                    Err(_) => vec![],
                }
            }
        })
        .into_iter()
        .flatten()
        .collect();

    let mut params = HashMap::with_capacity(metas.len());
    for (param, meta) in metas.into_iter().map(|meta| (GetSetAttr::new(&meta), meta)) {
        if params.insert(param, meta).is_some() {
            abort!(&attrs[0], "use of mutually exclusive GetSet attributes")
        }
    }

    params
}

pub fn produce(ast: &DeriveInput, global_params: &HashMap<GetSetAttr, Meta>) -> TokenStream2 {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let generated = fields.iter().map(|f| implement(f, global_params));
        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #(#generated)*
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        abort_call_site!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}

fn implement(field: &Field, global_params: &HashMap<GetSetAttr, Meta>) -> TokenStream2 {
    // Parse all local field attributes.
    let mut params = collect_attr(&field.attrs);

    // If field has skip attribute return immediately.
    if params.contains_key(&GetSetAttr::skip()) {
        return quote!();
    }

    // Merge global and local attributes.
    // Note: It is not possible to use extend, as it will only override
    // the values and not the keys. However, the attribute type is stored in
    // the key.
    for (key, value) in global_params.iter() {
        if !params.contains_key(key) {
            params.insert(key.clone(), value.clone());
        }
    }

    // Extract field name.
    let field_name = field
        .clone()
        .ident
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    // Extract fields doc comments.
    let doc = field.attrs.iter().filter(|v| {
        v.parse_meta()
            .map(|meta| meta.path().is_ident("doc"))
            .unwrap_or(false)
    });
    let doc = quote! { #(#doc)* };

    let generated = params.iter().map(|(param, meta)| {
        let visibility = parse_visibility(meta);

        let fn_name = if param.prefix.is_none()
            && (param.is_get())
            && param.suffix().is_empty()
            && field_name.to_string().starts_with("r#")
        {
            field_name.clone()
        } else {
            Ident::new(
                &format!("{}{}{}", param.prefix(), field_name.unraw(), param.suffix()),
                Span::call_site(),
            )
        };

        // In case of an Option<T>, it is necessary to unwrap the inner type T of it.
        let ty = match param.is_option() {
            true => extract_type_from_option(&field.ty),
            false => field.ty.clone(),
        };

        match param.mode {
            GetSetMode::Get => {
                quote! {
                    #doc
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.#field_name
                    }
                }
            }
            GetSetMode::GetCopy => {
                quote! {
                    #doc
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> #ty {
                        self.#field_name
                    }
                }
            }
            GetSetMode::GetOption => {
                quote! {
                    #doc
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> Option<&#ty> {
                        self.#field_name.as_ref()
                    }
                }
            }
            GetSetMode::Set => {
                quote! {
                    #doc
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                        self.#field_name = val;
                        self
                    }
                }
            }
            GetSetMode::SetOption => {
                quote! {
                    #doc
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                        self.#field_name = Some(val);
                        self
                    }
                }
            }
            GetSetMode::GetMut => {
                quote! {
                    #doc
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self) -> &mut #ty {
                        &mut self.#field_name
                    }
                }
            }
            GetSetMode::Skip => quote!(""),
        }
    });

    quote! { #(#generated)* }
}