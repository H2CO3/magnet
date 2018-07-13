//! Magnet, a JSON/BSON schema generator.
//!
//! This crate only contains the `#[derive(BsonSchema)]` proc-macro.
//! For documentation, please see the [`magnet_schema`][1] crate.
//!
//! [1]: https://docs.rs/magnet_schema

#![crate_type = "proc-macro"]
#![doc(html_root_url = "https://docs.rs/magnet_derive/0.3.0")]
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
extern crate proc_macro2;

mod tag;
mod case;
mod meta;
mod error;
mod generics;
mod codegen_field;
mod codegen_struct;
mod codegen_enum;
mod codegen_union;

use proc_macro::TokenStream;
use syn::{ DeriveInput, Data };
use error::Result;
use generics::GenericsExt;
use codegen_struct::*;
use codegen_enum::*;
use codegen_union::*;

/// The top-level entry point of this proc-macro. Only here to be exported
/// and to handle `Result::Err` return values by `panic!()`ing.
#[proc_macro_derive(BsonSchema, attributes(magnet))]
pub fn derive_bson_schema(input: TokenStream) -> TokenStream {
    impl_bson_schema(input).unwrap_or_else(|error| panic!("{}", error))
}

/// Implements `BsonSchema` for a given type based on its
/// recursively contained types in fields or variants.
fn impl_bson_schema(input: TokenStream) -> Result<TokenStream> {
    let parsed_ast: DeriveInput = syn::parse(input)?;
    let ty = parsed_ast.ident;
    let impl_ast = match parsed_ast.data {
        Data::Struct(s) => impl_bson_schema_struct(parsed_ast.attrs, s)?,
        Data::Enum(e) => impl_bson_schema_enum(parsed_ast.attrs, e)?,
        Data::Union(u) => impl_bson_schema_union(parsed_ast.attrs, u)?,
    };
    let (impbounds, tyargs, whbounds) = parsed_ast.generics.with_bson_schema();
    let generated = quote! {
        impl<#impbounds> ::magnet_schema::BsonSchema for #ty<#tyargs> #whbounds {
            fn bson_schema() -> ::bson::Document {
                #impl_ast
            }
        }
    };

    Ok(generated.into())
}
