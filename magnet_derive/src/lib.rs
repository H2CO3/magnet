//! Magnet, a JSON/BSON schema generator.
//!
//! This crate only contains the `#[derive(BsonSchema)]` proc-macro.
//! For documentation, please see the [`magnet_schema`][1] crate.
//!
//! [1]: https://docs.rs/magnet_schema

#![crate_type = "proc-macro"]
#![doc(html_root_url = "https://docs.rs/magnet_derive/0.1.1")]
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

mod case;
mod error;

use proc_macro::TokenStream;
use syn::{ DeriveInput, Data, DataStruct, DataEnum, DataUnion, Fields, Field };
use syn::{ Attribute, Meta, NestedMeta, MetaNameValue, Lit };
use syn::token::Comma;
use syn::punctuated::{ Punctuated, Pair };
use quote::Tokens;
use case::RenameRule;
use error::{ Error, Result };

/// The top-level entry point of this proc-macro. Only here to be exported
/// and to handle `Result::Err` return values by `panic!()`ing.
#[proc_macro_derive(BsonSchema, attributes(magnet))]
pub fn derive_bson_schema(input: TokenStream) -> TokenStream {
    impl_bson_schema(input).unwrap_or_else(|error| panic!("{}", error))
}

/// Implements `BsonSchema` for a given type based on its
/// recursively contained types in fields or variants.
/// TODO(H2CO3): handle generics
fn impl_bson_schema(input: TokenStream) -> Result<TokenStream> {
    let parsed_ast: DeriveInput = syn::parse(input)?;
    let type_name = parsed_ast.ident;
    let impl_ast = match parsed_ast.data {
        Data::Struct(s) => impl_bson_schema_struct(parsed_ast.attrs, s)?,
        Data::Enum(e) => impl_bson_schema_enum(parsed_ast.attrs, e)?,
        Data::Union(u) => impl_bson_schema_union(parsed_ast.attrs, u)?,
    };
    let generated = quote! {
        #[automatically_derived]
        impl ::magnet_schema::BsonSchema for #type_name {
            fn bson_schema() -> ::bson::Document {
                #impl_ast
            }
        }
    };

    Ok(generated.into())
}

/////////////
// Structs //
/////////////

/// Implements `BsonSchema` for a `struct`.
fn impl_bson_schema_struct(attrs: Vec<Attribute>, ast: DataStruct) -> Result<Tokens> {
    match ast.fields {
        Fields::Named(fields) => {
            impl_bson_schema_regular_struct(attrs, fields.named)
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
fn impl_bson_schema_regular_struct(attrs: Vec<Attribute>, fields: Punctuated<Field, Comma>) -> Result<Tokens> {
    let properties = &regular_struct_field_names(&attrs, &fields)?;
    let types = fields.iter().map(|field| &field.ty);

    Ok(quote! {
        doc! {
            "type": "object",
            "properties": {
                #(#properties: <#types as ::magnet_schema::BsonSchema>::bson_schema(),)*
            },
            "required": [ #(#properties,)* ],
            "additionalProperties": false,
        }
    })
}

/// Returns an iterator over the potentially-`#magnet[rename(...)]`d
/// fields of a regular struct with named fields.
fn regular_struct_field_names(attrs: &[Attribute], fields: &Punctuated<Field, Comma>) -> Result<Vec<String>> {
    let rename_all_str = serde_meta_name_value(attrs, "rename_all")?;
    let rename_all: Option<RenameRule> = match rename_all_str {
        Some(s) => Some(meta_value_as_str(&s)?.parse()?),
        None => None,
    };

    let iter = fields.iter().map(|field| {
        let name = field.ident.as_ref().ok_or_else(
            || Error::new("no name for named field?!")
        )?;

        let magnet_rename = magnet_meta_name_value(&field.attrs, "rename")?;
        let serde_rename = serde_meta_name_value(&field.attrs, "rename")?;
        let name = match magnet_rename.or(serde_rename) {
            Some(nv) => meta_value_as_str(&nv)?,
            None => rename_all.map_or(
                name.as_ref().into(),
                |rule| rule.apply_to_field(name.as_ref()),
            ),
        };

        Ok(name)
    });

    iter.collect()
}

/// Implements `BsonSchema` for a tuple `struct` with unnamed/numbered fields.
fn impl_bson_schema_tuple_struct(mut fields: Punctuated<Field, Comma>) -> Result<Tokens> {
    match fields.pop().map(Pair::into_value) {
        None => impl_bson_schema_unit_struct(), // 0 fields, equivalent to `()`
        Some(field) => match fields.len() {
            0 => {
                // 1 field, aka newtype - just delegate to the field's type
                let ty = field.ty;
                let tokens = quote! {
                    <#ty as ::magnet_schema::BsonSchema>::bson_schema()
                };
                Ok(tokens)
            },
            _ => {
                // more than 1 fields - treat it as if it was a tuple
                fields.push(field);

                let ty = fields.iter().map(|field| &field.ty);
                let tokens = quote! {
                    doc! {
                        "type": "array",
                        "items": [
                            #(<#ty as ::magnet_schema::BsonSchema>::bson_schema(),)*
                        ],
                        "additionalItems": false,
                    }
                };
                Ok(tokens)
            },
        }
    }
}

/// Implements `BsonSchema` for a unit `struct` with no fields.
fn impl_bson_schema_unit_struct() -> Result<Tokens> {
    Ok(quote!{ <() as ::magnet_schema::BsonSchema>::bson_schema() })
}

///////////
// Enums //
///////////

/// Implements `BsonSchema` for an `enum`.
/// TODO(H2CO3): implement me
fn impl_bson_schema_enum(_attrs: Vec<Attribute>, _ast: DataEnum) -> Result<Tokens> {
    Err(Error::new("`#[derive(BsonSchema)]` for `enum`s is not implemented"))
}

////////////
// Unions //
////////////

/// Implements `BsonSchema` for a `union`.
fn impl_bson_schema_union(_attrs: Vec<Attribute>, _ast: DataUnion) -> Result<Tokens> {
    Err(Error::new("`BsonSchema` can't be implemented for unions"))
}

/////////////////////
// General Helpers //
/////////////////////

/// Returns the inner, `...` part of the first `#[name(...)]` attribute
/// with the specified name (like `#[magnet(key ( = "value")?)]`).
/// TODO(H2CO3): check for duplicate arguments and bail out with an error
fn meta(attrs: &[Attribute], name: &str, key: &str) -> Option<Meta> {
    attrs.iter().filter_map(|attr| {
        let meta_list = match attr.interpret_meta()? {
            Meta::List(list) => {
                if list.ident.as_ref() == name {
                    list
                } else {
                    return None;
                }
            },
            _ => return None,
        };

        meta_list.nested.into_iter().filter_map(|nested_meta| {
            let meta = match nested_meta {
                NestedMeta::Meta(meta) => meta,
                _ => return None,
            };

            let ident = match meta {
                Meta::Word(ident) => ident,
                Meta::List(ref list) => list.ident,
                Meta::NameValue(ref name_value) => name_value.ident,
            };

            if ident.as_ref() == key {
                Some(meta)
            } else {
                None
            }
        })
        .next()
    })
    .next()
}

/// Search for an attribute, provided that it's a name-value pair.
fn meta_name_value(attrs: &[Attribute], name: &str, key: &str) -> Result<Option<MetaNameValue>> {
    match meta(attrs, name, key) {
        Some(Meta::NameValue(name_value)) => Ok(Some(name_value)),
        Some(_) => {
            let msg = format!("attribute must have form `#[{}({} = \"...\")]`", name, key);
            Err(Error::new(msg))
        },
        None => Ok(None),
    }
}

/// Search for a `Magnet` attribute, provided that it's a name-value pair.
fn magnet_meta_name_value(attrs: &[Attribute], key: &str) -> Result<Option<MetaNameValue>> {
    meta_name_value(attrs, "magnet", key)
}

/// Search for a `Serde` attribute, provided that it's a name-value pair.
fn serde_meta_name_value(attrs: &[Attribute], key: &str) -> Result<Option<MetaNameValue>> {
    meta_name_value(attrs, "serde", key)
}

/// Extracts a string value from an attribute value.
/// Returns `Err` if the value is not a `LitStr` nor a valid UTF-8 `LitByteStr`.
fn meta_value_as_str(nv: &MetaNameValue) -> Result<String> {
    match nv.lit {
        Lit::Str(ref string) => Ok(string.value()),
        Lit::ByteStr(ref string) => String::from_utf8(string.value()).map_err(Into::into),
        _ => Err(Error::new("attribute value must be a valid UTF-8 string")),
    }
}
