//! Helper functions for retrieving and parsing meta attributes.

use std::f64;
use syn::{ Attribute, Meta, NestedMeta, MetaNameValue, Lit };
use error::{ Error, Result };

/// Returns the inner, `...` part of the first `#[name(...)]` attribute
/// with the specified name (like `#[magnet(key ( = "value")?)]`).
/// TODO(H2CO3): check for duplicate arguments and bail out with an error
fn meta(attrs: &[Attribute], name: &str, key: &str) -> Option<Meta> {
    attrs.iter().filter_map(|attr| {
        let meta_list = match attr.interpret_meta()? {
            Meta::List(list) => {
                if list.ident == name {
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

            let ident = match meta.clone() {
                Meta::Word(ident) => ident,
                Meta::List(list) => list.ident,
                Meta::NameValue(name_value) => name_value.ident,
            };

            if ident == key {
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
fn name_value(attrs: &[Attribute], name: &str, key: &str) -> Result<Option<MetaNameValue>> {
    match meta(attrs, name, key) {
        Some(Meta::NameValue(name_value)) => Ok(Some(name_value)),
        Some(_) => {
            let msg = format!("attribute must have form `#[{}({} = \"...\")]`", name, key);
            Err(Error::new(msg))
        },
        None => Ok(None),
    }
}

/// Search for an attribute, provided that it's a single word.
fn has_meta_word(attrs: &[Attribute], name: &str, key: &str) -> Result<bool> {
    match meta(attrs, name, key) {
        Some(Meta::Word(_)) => Ok(true),
        Some(_) => {
            let msg = format!("attribute must have form `#[{}({})]`", name, key);
            Err(Error::new(msg))
        },
        None => Ok(false),
    }
}

/// Search for a `Magnet` attribute, provided that it's a name-value pair.
pub fn magnet_name_value(attrs: &[Attribute], key: &str) -> Result<Option<MetaNameValue>> {
    name_value(attrs, "magnet", key)
}

/// Search for a `Serde` attribute, provided that it's a name-value pair.
pub fn serde_name_value(attrs: &[Attribute], key: &str) -> Result<Option<MetaNameValue>> {
    name_value(attrs, "serde", key)
}

/// Search for a `Serde` attribute, provided that it's a single word.
pub fn has_serde_word(attrs: &[Attribute], key: &str) -> Result<bool> {
    has_meta_word(attrs, "serde", key)
}

/// Extracts a string value from an attribute value.
/// Returns `Err` if the value is not a `LitStr` nor a valid UTF-8 `LitByteStr`.
pub fn value_as_str(nv: &MetaNameValue) -> Result<String> {
    match nv.lit {
        Lit::Str(ref string) => Ok(string.value()),
        Lit::ByteStr(ref string) => String::from_utf8(string.value()).map_err(Into::into),
        _ => Err(Error::new("attribute value must be a valid UTF-8 string")),
    }
}

/// Extracts a floating-point value from an attribute value.
/// Returns an `Err` if the literal is not a valid floating-point
/// number or integer, and not a string that could be parsed as one.
#[allow(clippy::cast_precision_loss)]
pub fn value_as_num(nv: &MetaNameValue) -> Result<f64> {
    match nv.lit {
        Lit::Float(ref lit) => Ok(lit.value()),
        Lit::Int(ref lit) => {
            // Check if `f64` can represent this `u64`,
            // so the conversion would be lossless.
            let value = lit.value();
            let max_exact = 1 << f64::MANTISSA_DIGITS;
            if value < max_exact {
                Ok(value as f64)
            } else {
                Err(Error::new("Integer can't be exactly represented by `f64`"))
            }
        },
        Lit::Str(ref string) => string.value().parse().map_err(Into::into),
        Lit::ByteStr(ref string) => {
            String::from_utf8(string.value())
                .map_err(Into::into)
                .and_then(|s| s.parse().map_err(Into::into))
        },
        _ => Err(Error::new("attribute value must be a number")),
    }
}
