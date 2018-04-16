//! Helper functions for retrieving and parsing meta attributes.

use syn::{ Attribute, Meta, NestedMeta, MetaNameValue, Lit };
use error::{ Error, Result };

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
pub fn magnet_meta_name_value(attrs: &[Attribute], key: &str) -> Result<Option<MetaNameValue>> {
    meta_name_value(attrs, "magnet", key)
}

/// Search for a `Serde` attribute, provided that it's a name-value pair.
pub fn serde_meta_name_value(attrs: &[Attribute], key: &str) -> Result<Option<MetaNameValue>> {
    meta_name_value(attrs, "serde", key)
}

/// Extracts a string value from an attribute value.
/// Returns `Err` if the value is not a `LitStr` nor a valid UTF-8 `LitByteStr`.
pub fn meta_value_as_str(nv: &MetaNameValue) -> Result<String> {
    match nv.lit {
        Lit::Str(ref string) => Ok(string.value()),
        Lit::ByteStr(ref string) => String::from_utf8(string.value()).map_err(Into::into),
        _ => Err(Error::new("attribute value must be a valid UTF-8 string")),
    }
}
