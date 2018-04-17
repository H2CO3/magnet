//! Common part of codegen for `struct`s and `enum` variants.

use quote::Tokens;
use syn::{ Attribute, Field, Fields };
use syn::punctuated::{ Punctuated, Pair };
use syn::token::Comma;
use case::RenameRule;
use error::{ Error, Result };
use meta::*;

/// Implements `BsonSchema` for a struct or variant with the given fields.
pub fn impl_bson_schema_fields(attrs: Vec<Attribute>, fields: Fields) -> Result<Tokens> {
    match fields {
        Fields::Named(fields) => {
            impl_bson_schema_named_fields(attrs, fields.named)
        },
        Fields::Unnamed(fields) => {
            impl_bson_schema_indexed_fields(fields.unnamed)
        },
        Fields::Unit => {
            impl_bson_schema_unit_field()
        },
    }
}

/// Implements `BsonSchema` for a `struct` or variant with named fields.
fn impl_bson_schema_named_fields(attrs: Vec<Attribute>, fields: Punctuated<Field, Comma>) -> Result<Tokens> {
    let properties = &field_names(&attrs, &fields)?;
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
/// fields of a struct or variant with named fields.
fn field_names(attrs: &[Attribute], fields: &Punctuated<Field, Comma>) -> Result<Vec<String>> {
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

/// Implements `BsonSchema` for a tuple `struct` or variant,
/// with unnamed (numbered/indexed) fields.
fn impl_bson_schema_indexed_fields(mut fields: Punctuated<Field, Comma>) -> Result<Tokens> {
    match fields.pop().map(Pair::into_value) {
        None => impl_bson_schema_unit_field(), // 0 fields, equivalent to `()`
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

/// Implements `BsonSchema` for a unit `struct` or variant with no fields.
fn impl_bson_schema_unit_field() -> Result<Tokens> {
    Ok(quote!{ <() as ::magnet_schema::BsonSchema>::bson_schema() })
}
