// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides the `DebugStub` derive macro.
//!
//! The `DebugStub` derive macro can be used as a drop-in replacement for the
//! standard [`fmt::Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html)
//! when certain members of a struct or enum do not or cannot implement the
//! `fmt::Debug` trait themselves, but the containing struct or enum still wants
//! or needs to implement it.
//!
//! # Examples
//!
//! Using `DebugStub` with structs:
//!
//! ```
//! use debug_stub_derive::DebugStub;
//!
//! // A struct from an external crate which does not implement the `fmt::Debug`
//! // trait.
//! pub struct ExternalCrateStruct;
//!
//! // A struct in the current crate which wants to cleanly expose
//! // itself to the outside world with an implementation of `fmt::Debug`.
//! #[derive(DebugStub)]
//! pub struct PubStruct {
//!     a: bool,
//!     // Define a replacement debug serialization for the external struct.
//!     #[debug_stub = "ReplacementValue"]
//!     b: ExternalCrateStruct,
//! }
//!
//! assert_eq!(
//!     format!(
//!         "{:?}",
//!         PubStruct {
//!             a: true,
//!             b: ExternalCrateStruct,
//!         },
//!     ),
//!     "PubStruct { a: true, b: ReplacementValue }",
//! );
//! ```
//!
//! Using `DebugStub` with enums:
//!
//! ```
//! # use debug_stub_derive::DebugStub;
//! pub struct ExternalCrateStruct;
//!
//! #[derive(DebugStub)]
//! pub enum PubEnum {
//!     VariantA(u64, #[debug_stub = "ReplacementValue"] ExternalCrateStruct),
//! }
//!
//! assert_eq!(
//!     format!("{:?}", PubEnum::VariantA(42, ExternalCrateStruct)),
//!     "VariantA(42, ReplacementValue)",
//! );
//! ```
//!
//! Using `DebugStub` with `Option` and `Result` types:
//!
//! ```
//! # use debug_stub_derive::DebugStub;
//! pub struct ExternalCrateStruct;
//!
//! #[derive(DebugStub)]
//! pub struct PubStruct {
//!     #[debug_stub(some = "ReplacementSomeValue")]
//!     a: Option<ExternalCrateStruct>,
//!     #[debug_stub(ok = "ReplacementOkValue")]
//!     b: Result<ExternalCrateStruct, ()>,
//! }
//!
//! assert_eq!(
//!     format!(
//!         "{:?}",
//!         PubStruct {
//!             a: Some(ExternalCrateStruct),
//!             b: Ok(ExternalCrateStruct),
//!         },
//!     ),
//!     "PubStruct { a: Some(ReplacementSomeValue), b: Ok(ReplacementOkValue) }",
//! );
//! ```
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

extern crate proc_macro;

use proc_macro2::Span;
use quote::{quote, ToTokens as _};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned as _, Arm, Attribute, Data, DataEnum,
    DataStruct, DataUnion, DeriveInput, Expr, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident,
    Lit, LitStr, Meta, MetaList, MetaNameValue, NestedMeta, Pat, Stmt,
};

/// Implementation of the `#[derive(DebugStub)]` derive macro.
#[proc_macro_derive(DebugStub, attributes(debug_stub))]
pub fn derive_debug_stub(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_derive_serialize(&input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Central expansion function
fn expand_derive_serialize(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // check if there's an `#[debug_stub(ignore_generics)]` attribute
    let mut ignore_generics = false;
    for attr in &ast.attrs {
        let meta = match attr.parse_meta() {
            Ok(meta) if meta.path().is_ident("debug_stub") => meta,
            _ => continue,
        };

        if let Meta::List(inner) = &meta {
            for nested_meta in &inner.nested {
                match nested_meta {
                    NestedMeta::Meta(meta) if meta.path().is_ident("ignore_generics") => {
                        ignore_generics = true
                    }
                    _ => return Err(syn::Error::new(meta.span(), "expected `ignore_generics`")),
                }
            }
        } else {
            return Err(syn::Error::new(meta.span(), "expected `ignore_generics`"));
        }
    }

    let mut generics_debug_bounded = ast.generics.clone();
    if !ignore_generics {
        for generic_param in &mut generics_debug_bounded.params {
            if let syn::GenericParam::Type(generic_type_param) = generic_param {
                generic_type_param
                    .bounds
                    .push(parse_quote!(::core::fmt::Debug));
            }
        }
    }

    match &ast.data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(fields) => {
                let stmts = generate_field_stmts(&fields)?;
                Ok(implement_named_fields_struct_debug(
                    &ast.ident,
                    &generics_debug_bounded,
                    &stmts,
                ))
            }
            Fields::Unnamed(fields) => {
                let stmts = generate_tuple_field_stmts(&fields)?;
                Ok(implement_unnamed_fields_struct_debug(
                    &ast.ident,
                    &generics_debug_bounded,
                    &stmts,
                ))
            }
            Fields::Unit => Ok(implement_unit_struct_debug(
                &ast.ident,
                &generics_debug_bounded,
            )),
        },
        Data::Enum(DataEnum { variants, .. }) => Ok(implement_enum_debug(
            &ast.ident,
            &generics_debug_bounded,
            &variants
                .iter()
                .map(|variant| generate_arm(&ast.ident, variant))
                .collect::<syn::Result<Vec<_>>>()?,
        )),
        Data::Union(DataUnion { union_token, .. }) => Err(syn::Error::new_spanned(
            union_token,
            "expected struct or enum",
        )),
    }
}

/// Generates named fields struct Debug impl (`MyStruct { field1: ..., field2: ... }`) from a given
/// list of formatter statements (`f.field("field1", ...)`, `f.field("field2", ...)`)
fn implement_named_fields_struct_debug(
    ident: &Ident,
    generics: &Generics,
    stmts: &[Stmt],
) -> proc_macro2::TokenStream {
    let name = ident.to_string();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!(
        impl #impl_generics ::core::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let mut f = f.debug_struct(#name);
                #(#stmts)*
                f.finish()
            }
        }
    )
}

/// Generates unnamed fields struct Debug impl (`MyStruct(..., ...)`) from a given
/// list of formatter statements (`f.field(...)`, `f.field(...)`)
fn implement_unnamed_fields_struct_debug(
    ident: &Ident,
    generics: &Generics,
    stmts: &[Stmt],
) -> proc_macro2::TokenStream {
    let name = ident.to_string();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!(
        impl #impl_generics ::core::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let mut f = f.debug_tuple(#name);
                #(#stmts)*
                f.finish()
            }
        }
    )
}

/// Generates unit struct Debug impl (`MyStruct`)
fn implement_unit_struct_debug(ident: &Ident, generics: &Generics) -> proc_macro2::TokenStream {
    let name = ident.to_string();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!(
        impl #impl_generics ::core::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(#name)
            }
        }
    )
}

/// Generates enum Debug impl from given match arm list (`MyStruct::A => { ... }`,
/// `MyStruct::B => { ... }`)
fn implement_enum_debug(
    ident: &Ident,
    generics: &Generics,
    arms: &[Arm],
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::core::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

/// Generates Formatter statements for a named fields struct like `f.field("a", self.a)`
fn generate_field_stmts(fields: &FieldsNamed) -> syn::Result<Vec<Stmt>> {
    fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let expr = parse_quote!(self.#ident);
            let name = ident.to_string();
            let (_, stmt) = extract_value_attr(&expr, &field.attrs, Some(name))?;
            Ok(stmt)
        })
        .collect()
}

/// Generates Formatter statements for a tuple struct like `f.field(self.0)`
fn generate_tuple_field_stmts(fields: &FieldsUnnamed) -> syn::Result<Vec<Stmt>> {
    fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let index = syn::Index::from(index);
            let expr = parse_quote!(self.#index);
            let (_, stmt) = extract_value_attr(&expr, &field.attrs, None)?;
            Ok(stmt)
        })
        .collect()
}

/// Generates a single match arm for an enum Debug impl
fn generate_arm(ident: &Ident, variant: &syn::Variant) -> syn::Result<Arm> {
    let variant_ident = &variant.ident;
    let variant_name = variant_ident.to_string();

    match &variant.fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let fields = named
                .iter()
                .map(|field| {
                    let ident = field
                        .ident
                        .clone()
                        .expect("Tuple struct variant has unnamed fields");
                    (ident.clone(), &field.attrs[..], Some(ident.to_string()))
                })
                .collect();
            let (pats, stmts) = generate_enum_variant_fields(fields)?;

            Ok(parse_quote! {
                #ident::#variant_ident { #(#pats),* } => {
                    let mut f = f.debug_struct(#variant_name);
                    #(#stmts)*
                    f.finish()
                }
            })
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let fields = unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    (
                        Ident::new(&format!("tuple_{}", index), Span::call_site()),
                        &field.attrs[..],
                        None,
                    )
                })
                .collect();
            let (pats, stmts) = generate_enum_variant_fields(fields)?;

            Ok(parse_quote! {
                #ident::#variant_ident( #(#pats),* ) => {
                    let mut f = f.debug_tuple(#variant_name);
                    #(#stmts)*
                    f.finish()
                }
            })
        }
        Fields::Unit => Ok(parse_quote! {
            #ident::#variant_ident => f.write_str(#variant_name),
        }),
    }
}

/// Generates match arm pattern and Formatter statements for an enum variant
fn generate_enum_variant_fields(
    fields: Vec<(Ident, &[Attribute], Option<String>)>,
) -> syn::Result<(Vec<Pat>, Vec<Stmt>)> {
    let mut pats = vec![];
    let mut unused_fields = false;

    let stmts = fields
        .into_iter()
        .map(|(ident, attrs, name)| {
            let unnamed = name.is_none();
            let (ident_used, stmt) = extract_value_attr(&parse_quote!(#ident), attrs, name)?;

            if ident_used {
                pats.push(parse_quote!(#ident));
            } else if unnamed {
                // Skip unused tuple fields to avoid "unused variable" warnings
                pats.push(parse_quote!(_));
            } else {
                // Skip unused struct fields to avoid "unused variable" warnings
                unused_fields = true;
            }

            Ok(stmt)
        })
        .collect::<syn::Result<Vec<_>>>()?;

    if unused_fields {
        pats.push(parse_quote!(..));
    }

    Ok((pats, stmts))
}

/// Generates a single Formatter statement from given field and attributes. Also returns whether the
/// field value is actually being used in the statement
fn extract_value_attr(
    expr: &Expr,
    attrs: &[Attribute],
    name: Option<String>,
) -> syn::Result<(bool, Stmt)> {
    for attr in attrs {
        let meta = match attr.parse_meta() {
            Ok(meta) if meta.path().is_ident("debug_stub") => meta,
            _ => continue,
        };

        match meta {
            // `#[debug_stub]`
            Meta::Path(path) => {
                return Err(syn::Error::new_spanned(
                    path,
                    "expected `List` or `NameValue`",
                ));
            }
            // `#[debug_stub(key1 = val1, key2 = val2)]`
            Meta::List(MetaList { nested, .. }) => {
                return match extract_named_value_attrs(nested.iter()) {
                    (None, None, Some(some)) => Ok((true, implement_some_attr(&some, name, expr))),
                    (Some(ok), Some(err), None) => {
                        Ok((true, implement_result_attr(&ok, &err, name, expr)))
                    }
                    (Some(ok), None, None) => Ok((true, implement_ok_attr(&ok, name, expr))),
                    (None, Some(err), None) => Ok((true, implement_err_attr(&err, name, expr))),
                    _ => Err(syn::Error::new_spanned(
                        nested,
                        "expected `some = _`, `ok = _`, `err = _`, or `ok = _, err = _`",
                    )),
                };
            }
            // `#[debug_stub = "literal"]`
            Meta::NameValue(MetaNameValue { lit, .. }) => {
                let lit = syn::parse2::<LitStr>(lit.to_token_stream())?;
                return Ok((false, implement_replace_attr(name, &lit.value())));
            }
        }
    }

    Ok(match name {
        Some(name) => (true, parse_quote!(f.field(#name, &#expr);)),
        None => (true, parse_quote!(f.field(&#expr);)),
    })
}

/// Extracts the `ok = "..."`, `err = "..."`, and `some = "..."` attributes, if present
fn extract_named_value_attrs<'a>(
    nested: impl Iterator<Item = &'a NestedMeta>,
) -> (Option<String>, Option<String>, Option<String>) {
    let (mut ok, mut err, mut some) = (None, None, None);

    for nested in nested {
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(lit),
            ..
        })) = nested
        {
            if path.is_ident("some") {
                some = Some(lit.value());
            } else if path.is_ident("ok") {
                ok = Some(lit.value());
            } else if path.is_ident("err") {
                err = Some(lit.value());
            }
        }
    }

    (ok, err, some)
}

/// Generates `f.field()` Formatter statement for `#[debug_stub = "..."]`
fn implement_replace_attr(name: Option<String>, value: &str) -> Stmt {
    if let Some(name) = name {
        parse_quote!(f.field(#name, &format_args!("{}", #value));)
    } else {
        parse_quote!(f.field(&format_args!("{}", #value));)
    }
}

/// Generates `f.field()` Formatter statement for `#[debug_stub(some = "...")]`
fn implement_some_attr(some: &str, name: Option<String>, expr: &Expr) -> Stmt {
    if let Some(name) = name {
        parse_quote! {
            if #expr.is_some() {
                f.field(#name, &Some::<_>(format_args!("{}", #some)));
            } else {
                f.field(#name, &format_args!("None"));
            }
        }
    } else {
        parse_quote! {
            if #expr.is_some() {
                f.field(&Some::<_>(format_args!("{}", #some)));
            } else {
                f.field(&format_args!("None"));
            }
        }
    }
}

/// Generates `f.field()` Formatter statement for `#[debug_stub(ok = "...", err = "...")]`
fn implement_result_attr(ok: &str, err: &str, name: Option<String>, expr: &Expr) -> Stmt {
    if let Some(name) = name {
        parse_quote! {
            if #expr.is_ok() {
                f.field(#name, &Ok::<_, ()>(format_args!("{}", #ok)));
            } else {
                f.field(#name, &Err::<(), _>(format_args!("{}", #err)));
            }
        }
    } else {
        parse_quote! {
            if #expr.is_ok() {
                f.field(&Ok::<_, ()>(format_args!("{}", #ok)));
            } else {
                f.field(&Err::<(), _>(format_args!("{}", #err)));
            }
        }
    }
}

/// Generates `f.field()` Formatter statement for `#[debug_stub(ok = "...")]`
fn implement_ok_attr(ok: &str, name: Option<String>, expr: &Expr) -> Stmt {
    if let Some(name) = name {
        parse_quote! {
            if #expr.is_err() {
                f.field(#name, &Err::<(), _>(#expr.as_ref().err().unwrap()));
            } else {
                f.field(#name, &Ok::<_, ()>(format_args!("{}", #ok)));
            }
        }
    } else {
        parse_quote! {
            if #expr.is_err() {
                f.field(&Err::<(), _>(#expr.as_ref().err().unwrap()));
            } else {
                f.field(&Ok::<_, ()>(format_args!("{}", #ok)));
            }
        }
    }
}

/// Generates `f.field()` Formatter statement for `#[debug_stub(err = "...")]`
fn implement_err_attr(err: &str, name: Option<String>, expr: &Expr) -> Stmt {
    if let Some(name) = name {
        parse_quote! {
            if #expr.is_ok() {
                f.field(#name, &Ok::<_, ()>(#expr.as_ref().ok().unwrap()));
            } else {
                f.field(#name, &Err::<(), _>(format_args!("{}", #err)));
            }
        }
    } else {
        parse_quote! {
            if #expr.is_ok() {
                f.field(&Ok::<_, ()>(#expr.as_ref().ok().unwrap()));
            } else {
                f.field(&Err::<(), _>(format_args!("{}", #err)));
            }
        }
    }
}
