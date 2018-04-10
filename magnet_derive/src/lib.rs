//! Magnet, a JSON/BSON schema generator.

#![crate_type = "proc-macro"]
#![doc(html_root_url = "https://docs.rs/magnet_derive/0.1.0")]
#![deny(missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications,
        /* missing_docs (https://github.com/rust-lang/rust/issues/42008) */)]
#![cfg_attr(feature = "cargo-clippy",
            allow(single_match, match_same_arms, match_ref_pats,
                  clone_on_ref_ptr, needless_pass_by_value))]
#![cfg_attr(feature = "cargo-clippy",
            deny(wrong_pub_self_convention, used_underscore_binding,
                 stutter, similar_names, pub_enum_variant_names,
                 missing_docs_in_private_items,
                 non_ascii_literal, unicode_not_nfc,
                 result_unwrap_used, option_unwrap_used,
                 option_map_unwrap_or_else, option_map_unwrap_or, filter_map,
                 shadow_unrelated, shadow_reuse, shadow_same,
                 int_plus_one, string_add_assign, if_not_else,
                 invalid_upcast_comparisons,
                 cast_precision_loss,
                 cast_possible_wrap, cast_possible_truncation,
                 mutex_integer, mut_mut, items_after_statements,
                 print_stdout, mem_forget, maybe_infinite_iter))]

#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{ DeriveInput, Data, DataStruct, DataEnum, Fields, Field };
use syn::token::Comma;
use syn::punctuated::Punctuated;
use quote::Tokens;

/// Implements `BsonSchema` for a given type based on its
/// recursively contained types in fields or variants.
/// TODO(H2CO3): handle generics
#[proc_macro_derive(BsonSchema)]
pub fn derive_bson_schema(input: TokenStream) -> TokenStream {
    let parsed_ast: DeriveInput = syn::parse(input).expect("couldn't parse derive input");
    let type_name = parsed_ast.ident;
    let impl_ast = match parsed_ast.data {
        Data::Struct(s) => impl_bson_schema_struct(s),
        Data::Enum(e) => impl_bson_schema_enum(e),
        Data::Union(_) => panic!("`BsonSchema` can't be implemented for unions"),
    };
    let generated = quote! {
        #[automatically_derived]
        impl ::magnet_schema::BsonSchema for #type_name {
            fn bson_schema() -> ::bson::Document {
                #impl_ast
            }
        }
    };

    generated.into()
}

/// Implements `BsonSchema` for a `struct`.
fn impl_bson_schema_struct(ast: DataStruct) -> Tokens {
    match ast.fields {
        Fields::Named(fields) => {
            impl_bson_schema_regular_struct(fields.named)
        },
        Fields::Unnamed(fields) => {
            impl_bson_schema_tuple_struct(fields.unnamed)
        },
        Fields::Unit => {
            impl_bson_schema_unit_struct()
        },
    }
}

/// Implements `BsonSchema` for a regular `struct` with named fields.
fn impl_bson_schema_regular_struct(fields: Punctuated<Field, Comma>) -> Tokens {
    // TODO(H2CO3): figure out and handle `"id"`-vs.-`"_id"`!!!
    // TODO(H2CO3): handle `serde(rename)`, `serde(rename_all)`, `serde(skip)`, etc.
    let property_names = fields.iter().map(
        |field| field.ident.as_ref().expect("no name for named field?!").as_ref()
    );
    let required_names = fields.iter().map(
        |field| field.ident.as_ref().expect("no name for named field?!").as_ref()
    );
    let types = fields.iter().map(|field| &field.ty);

    quote! {
        doc! {
            "type": "object",
            "properties": { #(#property_names: #types::bson_schema(),)* },
            "required": [ #(#required_names,)* ],
            "additionalProperties": false,
        }
    }
}

/// Implements `BsonSchema` for a tuple `struct` with unnamed/numbered fields.
/// TODO(H2CO3): implement me
fn impl_bson_schema_tuple_struct(fields: Punctuated<Field, Comma>) -> Tokens {
    unimplemented!()
}

/// Implements `BsonSchema` for a unit `struct` with no fields.
fn impl_bson_schema_unit_struct() -> Tokens {
    quote! {
        doc! {
            "type": ["array", "null"],
            "maxItems": 0,
        }
    }
}

/// Implements `BsonSchema` for an `enum`.
/// TODO(H2CO3): implement me
fn impl_bson_schema_enum(ast: DataEnum) -> Tokens {
    unimplemented!()
}
