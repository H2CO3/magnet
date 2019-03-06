//! Code generation for `enum`s.

use syn::{ Attribute, DataEnum, Variant, Fields };
use proc_macro2::TokenStream;
use error::{ Error, Result };
use case::RenameRule;
use tag::SerdeEnumTag;
use codegen_field::*;
use meta;

/// Implements `BsonSchema` for an `enum`.
/// TODO(H2CO3): implement me
pub fn impl_bson_schema_enum(attrs: Vec<Attribute>, ast: DataEnum) -> Result<TokenStream> {
    let rename_all_str = meta::serde_name_value(&attrs, "rename_all")?;
    let rename_all: Option<RenameRule> = match rename_all_str {
        Some(s) => Some(meta::value_as_str(&s)?.parse()?),
        None => None,
    };
    let doc = doc_meta(&attrs).and_then(|doc| meta_value_as_str(&doc).ok());
    let doc = quote! {
        "description": #doc
    };
    let tagging = SerdeEnumTag::from_attrs(&attrs)?;

    let variants: Vec<_> = ast.variants
        .into_iter()
        .map(|variant| variant_schema(variant, rename_all, &tagging))
        .collect::<Result<_>>()?;

    let tokens = quote! {
        doc! {
            #doc,
            "anyOf": [ #(#variants,)* ]
        }
    };

    Ok(tokens)
}

/// Generates a `BsonSchema` for a single `enum` variant.
fn variant_schema(
    variant: Variant,
    rename_all: Option<RenameRule>,
    tagging: &SerdeEnumTag,
) -> Result<TokenStream> {
    // check for renaming directive attribute
    if meta::magnet_name_value(&variant.attrs, "rename")?.is_some() {
        return Err(Error::new("`#[magnet(rename = \"...\")]` no longer exists"))
    }

    let rename = meta::serde_name_value(&variant.attrs, "rename")?;
    let variant_name = match rename {
        Some(nv) => meta::value_as_str(&nv)?,
        None => rename_all.map_or_else(
            || variant.ident.to_string(),
            |rule| rule.apply_to_variant(variant.ident.to_string()),
        ),
    };

    match *tagging {
        SerdeEnumTag::Untagged => {
            impl_bson_schema_fields(&variant.attrs, variant.fields)
        }
        SerdeEnumTag::Adjacent {
            ref tag, ref content
        } => match variant.fields {
            Fields::Unit => adjacently_tagged_unit_variant_schema(
                &variant_name,
                tag,
            ),
            _ => adjacently_tagged_other_variant_schema(
                &variant.attrs,
                &variant_name,
                tag,
                content,
                variant.fields,
            ),
        },
        SerdeEnumTag::Internal(ref tag) => match variant.fields {
            Fields::Unit => internally_tagged_unit_variant_schema(
                &variant_name,
                tag,
            ),
            _ => internally_tagged_other_variant_schema(
                &variant.attrs,
                &variant_name,
                tag,
                variant.fields,
            ),
        },
        SerdeEnumTag::External => match variant.fields {
            Fields::Unit => externally_tagged_unit_variant_schema(&variant_name),
            _ => externally_tagged_other_variant_schema(
                &variant.attrs,
                &variant_name,
                variant.fields,
            ),
        },
    }
}

/// Generates a schema for a unit variant
/// if the containing enum is adjacently tagged.
fn adjacently_tagged_unit_variant_schema(variant_name: &str, tag: &str) -> Result<TokenStream> {
    let tokens = quote! {
        doc! {
            "type": "object",
            "additionalProperties": false,
            "required": [ #tag ],
            "properties": {
                #tag: { "enum": [ #variant_name ] },
            },
        }
    };
    Ok(tokens)
}

/// Generates a schema for a non-unit (newtype, tuple, or struct) variant
/// if the containing enum is adjacently tagged.
fn adjacently_tagged_other_variant_schema(
    attrs: &[Attribute],
    variant_name: &str,
    tag: &str,
    content: &str,
    fields: Fields,
) -> Result<TokenStream> {
    let variant_schema = impl_bson_schema_fields(attrs, fields)?;
    let tokens = quote! {
        doc! {
            "type": "object",
            "additionalProperties": false,
            "required": [ #tag, #content ],
            "properties": {
                #tag: { "enum": [ #variant_name ] },
                #content: #variant_schema,
            },
        }
    };
    Ok(tokens)
}

/// Generates a schema for a unit variant if the containing enum is
/// internally tagged. Incidentally, the representation is exactly
/// the same as that of the adjacently-tagged version.
fn internally_tagged_unit_variant_schema(variant_name: &str, tag: &str) -> Result<TokenStream> {
    adjacently_tagged_unit_variant_schema(variant_name, tag)
}

/// Generates a schema for a non-unit (newtype or struct)
/// variant if the containing enum is internally tagged.
fn internally_tagged_other_variant_schema(
    attrs: &[Attribute],
    variant: &str,
    tag: &str,
    fields: Fields,
) -> Result<TokenStream> {
    let tag_extra = TagExtra { tag, variant };

    impl_bson_schema_fields_extra(attrs, fields, tag_extra.into())
}

/// Generates a schema for a unit variant
/// if the containing enum is externally tagged.
fn externally_tagged_unit_variant_schema(variant_name: &str) -> Result<TokenStream> {
    let tokens = quote! {
        doc! {
            "enum": [ #variant_name ],
        }
    };
    Ok(tokens)
}

/// Generates a schema for a non-unit (newtype, tuple, or struct)
/// variant if the containing enum is externally tagged.
fn externally_tagged_other_variant_schema(
    attrs: &[Attribute],
    variant_name: &str,
    fields: Fields,
) -> Result<TokenStream> {
    let variant_schema = impl_bson_schema_fields(attrs, fields)?;

    let tokens = quote! {
        doc! {
            "type": "object",
            "additionalProperties": false,
            "required": [ #variant_name ],
            "properties": {
                #variant_name: #variant_schema
            },
        }
    };
    Ok(tokens)
}
