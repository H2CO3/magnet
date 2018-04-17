//! Code generation for `enum`s.

use quote::Tokens;
use syn::{ Attribute, DataEnum, Variant, Fields };
use error::{ Error, Result };
use case::RenameRule;
use tag::SerdeEnumTag;
use codegen_field::impl_bson_schema_fields;
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

    match *tagging {
        SerdeEnumTag::Untagged => impl_bson_schema_fields(&[], variant.fields),
        SerdeEnumTag::Adjacent { ref tag, ref content } => {
            match variant.fields {
                Fields::Unit => adjacently_tagged_unit_variant_schema(
                    &variant_name,
                    tag,
                ),
                fields => adjacently_tagged_other_variant_schema(
                    &variant_name,
                    tag,
                    content,
                    fields,
                ),
            }
        }
        SerdeEnumTag::Internal(_) => Err(Error::new("internally-tagged enums are unimplemented")),
        SerdeEnumTag::External => match variant.fields {
            Fields::Unit => externally_tagged_unit_variant_schema(&variant_name),
            fields => externally_tagged_other_variant_schema(&variant_name, fields),
        },
    }
}

fn adjacently_tagged_unit_variant_schema(variant_name: &str, tag: &str) -> Result<Tokens> {
    let tokens = quote! {
        doc! {
            "type": "object",
            "properties": {
                #tag: { "enum": [ #variant_name ] },
            },
            "required": [ #tag ],
            "additionalProperties": false,
        }
    };
    Ok(tokens)
}

fn adjacently_tagged_other_variant_schema(
    variant_name: &str,
    tag: &str,
    content: &str,
    fields: Fields,
) -> Result<Tokens> {
    let variant_schema = impl_bson_schema_fields(&[], fields)?;
    let tokens = quote! {
        doc! {
            "type": "object",
            "properties": {
                #tag: { "enum": [ #variant_name ] },
                #content: #variant_schema,
            },
            "required": [ #tag, #content ],
            "additionalProperties": false,
        }
    };
    Ok(tokens)
}

fn externally_tagged_unit_variant_schema(variant_name: &str) -> Result<Tokens> {
    let tokens = quote! {
        doc! {
            "type": "string",
            "enum": [ #variant_name ],
        }
    };
    Ok(tokens)
}

fn externally_tagged_other_variant_schema(variant_name: &str, fields: Fields) -> Result<Tokens> {
    let variant_schema = impl_bson_schema_fields(&[], fields)?;

    let tokens = quote! {
        doc! {
            "type": "object",
            "properties": {
                #variant_name: #variant_schema
            },
            "required": [ #variant_name ],
            "additionalProperties": false,
        }
    };
    Ok(tokens)
}
