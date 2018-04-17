//! Code generation for `enum`s.

use quote::Tokens;
use syn::{ Attribute, DataEnum, Variant };
use error::{ Error, Result };
use case::RenameRule;
use tag::SerdeEnumTag;
use meta::*;

/// Implements `BsonSchema` for an `enum`.
/// TODO(H2CO3): implement me
pub fn impl_bson_schema_enum(attrs: Vec<Attribute>, ast: DataEnum) -> Result<Tokens> {
    let rename_all_str = serde_meta_name_value(&attrs, "rename_all")?;
    let rename_all: Option<RenameRule> = match rename_all_str {
        Some(s) => Some(meta_value_as_str(&s)?.parse()?),
        None => None,
    };
    let tagging = SerdeEnumTag::from_attrs(&attrs)?;

    let variants: Vec<_> = ast.variants
        .into_iter()
        .map(|variant| variant_schema(variant, rename_all, &tagging))
        .collect::<Result<_>>()?;

    let tokens = quote! {
        doc! {
            "oneOf": [
                #(#variants,)*
            ]
        }
    };

    Ok(tokens)
}

/// Generates a `BsonSchema` for a single `enum` variant.
fn variant_schema(
    variant: Variant,
    rename_all: Option<RenameRule>,
    tagging: &SerdeEnumTag,
) -> Result<Tokens> {
    // check for renaming directive attribute
    let magnet_rename = magnet_meta_name_value(&variant.attrs, "rename")?;
    let serde_rename = serde_meta_name_value(&variant.attrs, "rename")?;
    let variant_name = match magnet_rename.or(serde_rename) {
        Some(nv) => meta_value_as_str(&nv)?,
        None => rename_all.map_or(
            variant.ident.as_ref().into(),
            |rule| rule.apply_to_variant(variant.ident.as_ref()),
        ),
    };

    Err(Error::new("unimplemented"))
}
