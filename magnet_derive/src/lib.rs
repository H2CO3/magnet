//! Magnet, a JSON/BSON schema generator.
//!
//! This crate only contains the `#[derive(BsonSchema)]` proc-macro.
//! For documentation, please see the [`magnet_schema`][1] crate.
//!
//! [1]: https://docs.rs/magnet_schema

#![crate_type = "proc-macro"]
#![doc(html_root_url = "https://docs.rs/magnet_derive/0.8.0")]
#![deny(missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications,
        /* missing_docs (https://github.com/rust-lang/rust/issues/42008) */)]
#![allow(clippy::single_match, clippy::match_same_arms, clippy::match_ref_pats,
         clippy::clone_on_ref_ptr, clippy::needless_pass_by_value)]
#![deny(clippy::wrong_pub_self_convention, clippy::used_underscore_binding,
        clippy::stutter, clippy::similar_names, clippy::pub_enum_variant_names,
        clippy::missing_docs_in_private_items,
        clippy::non_ascii_literal, clippy::unicode_not_nfc,
        clippy::result_unwrap_used, clippy::option_unwrap_used,
        clippy::option_map_unwrap_or_else, clippy::option_map_unwrap_or, clippy::filter_map,
        clippy::shadow_unrelated, clippy::shadow_reuse, clippy::shadow_same,
        clippy::int_plus_one, clippy::string_add_assign, clippy::if_not_else,
        clippy::invalid_upcast_comparisons,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap, clippy::cast_possible_truncation,
        clippy::mutex_integer, clippy::mut_mut, clippy::items_after_statements,
        clippy::print_stdout, clippy::mem_forget, clippy::maybe_infinite_iter)]

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
    let generics = parsed_ast.generics;
    let (impl_gen, ty_gen, where_cls) = generics.split_and_augment_for_impl();
    let generated = quote! {
        impl #impl_gen ::magnet_schema::BsonSchema for #ty #ty_gen #where_cls {
            fn bson_schema() -> ::bson::Document {
                #impl_ast
            }
        }
    };

    Ok(generated.into())
}
