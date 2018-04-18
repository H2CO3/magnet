//! Common part of codegen for `struct`s and `enum` variants.

use quote::Tokens;
use syn::{ Attribute, Field, Fields, MetaNameValue };
use syn::punctuated::{ Punctuated, Pair };
use syn::token::Comma;
use case::RenameRule;
use error::{ Error, Result };
use meta::*;

/// Describes the extra field corresponding to an internally-tagged enum's tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagExtra<'a> {
    /// The name of the tag itself, which will be the key in the resulting map.
    pub tag: &'a str,
    /// The name of the enum variant, which will be the corresponding value.
    pub variant: &'a str,
}

/// Implements `BsonSchema` for a struct or variant with the given fields.
pub fn impl_bson_schema_fields(attrs: &[Attribute], fields: Fields) -> Result<Tokens> {
    impl_bson_schema_fields_extra(attrs, fields, None)
}

/// Similar to `impl_bson_schema_fields`, but accepts an additional
/// internal tag descriptor. Useful for implementing `enum`s.
pub fn impl_bson_schema_fields_extra(
    attrs: &[Attribute],
    fields: Fields,
    extra: Option<TagExtra>
) -> Result<Tokens> {
    match fields {
        Fields::Named(fields) => {
            impl_bson_schema_named_fields(attrs, fields.named, extra)
        },
        Fields::Unnamed(fields) => {
            impl_bson_schema_indexed_fields(fields.unnamed, extra)
        },
        Fields::Unit => {
            assert!(extra.is_none(), "internally-tagged unit should've been handled");
            impl_bson_schema_unit_field()
        },
    }
}

/// Implements `BsonSchema` for a `struct` or variant with named fields.
fn impl_bson_schema_named_fields(
    attrs: &[Attribute],
    fields: Punctuated<Field, Comma>,
    extra: Option<TagExtra>,
) -> Result<Tokens> {
    let properties = &field_names(attrs, &fields)?;
    let defs: Vec<_> = fields.iter().map(field_def).collect::<Result<_>>()?;
    let tokens = if let Some(TagExtra { tag, variant }) = extra {
        quote! {
            doc! {
                "type": "object",
                "additionalProperties": false,
                "required": [ #tag, #(#properties,)* ],
                "properties": {
                    #tag: { "enum": [ #variant ] },
                    #(#properties: #defs,)*
                },
            }
        }
    } else {
        quote! {
            doc! {
                "type": "object",
                "additionalProperties": false,
                "required": [ #(#properties,)* ],
                "properties": {
                    #(#properties: #defs,)*
                },
            }
        }
    };

    Ok(tokens)
}

/// Generates code for the value part of a key-value pair in a schema,
/// corresponding to a single named struct field.
/// TODO(H2CO3): check if field is numeric if bounded?
fn field_def(field: &Field) -> Result<Tokens> {
    let ty = &field.ty;
    let min_incl = magnet_meta_name_value(&field.attrs, "min_incl")?;
    let min_excl = magnet_meta_name_value(&field.attrs, "min_excl")?;
    let max_incl = magnet_meta_name_value(&field.attrs, "max_incl")?;
    let max_excl = magnet_meta_name_value(&field.attrs, "max_excl")?;
    let lower = bounds_from_meta(min_incl, min_excl)?;
    let upper = bounds_from_meta(max_incl, max_excl)?;

    Ok(quote! {
        ::magnet_schema::support::extend_schema_with_bounds(
            <#ty as ::magnet_schema::BsonSchema>::bson_schema(),
            ::magnet_schema::support::Bounds {
                lower: #lower,
                upper: #upper,
            },
        )
    })
}

/// Parses meta attrs into quoted `Bound`s.
fn bounds_from_meta(incl: Option<MetaNameValue>, excl: Option<MetaNameValue>) -> Result<Tokens> {
    // Inclusive takes precedence over exclusive (form a union).
    // TODO(H2CO3): this could be the other way around (when both
    // inclusive and exclusive bounds specified, form an intersection)
    // -- I'm not sure, which one makes more sense? Or maybe an error?
    if let Some(nv) = incl {
        let value = meta_value_as_num(&nv)?;

        Ok(quote! {
            ::magnet_schema::support::Bound::Inclusive(#value)
        })
    } else if let Some(nv) = excl {
        let value = meta_value_as_num(&nv)?;

        Ok(quote! {
            ::magnet_schema::support::Bound::Exclusive(#value)
        })
    } else {
        Ok(quote! {
            ::magnet_schema::support::Bound::Unbounded
        })
    }
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
fn impl_bson_schema_indexed_fields(
    mut fields: Punctuated<Field, Comma>,
    extra: Option<TagExtra>,
) -> Result<Tokens> {
    if extra.is_some() && fields.len() != 1 {
        return Err(Error::new("internal tagging not usable with tuple variant"))
    }

    match fields.pop().map(Pair::into_value) {
        None => impl_bson_schema_unit_field(), // 0 fields, equivalent to `()`
        Some(field) => match fields.len() {
            0 => {
                // 1 field, aka newtype - just delegate to the field's type
                let ty = field.ty;
                let tokens = if let Some(TagExtra { tag, variant }) = extra {
                    quote! {
                        ::magnet_schema::support::extend_schema_with_tag(
                            <#ty as ::magnet_schema::BsonSchema>::bson_schema(),
                            #tag,
                            #variant,
                        )
                    }
                } else {
                    quote! {
                        <#ty as ::magnet_schema::BsonSchema>::bson_schema()
                    }
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
                        "additionalItems": false,
                        "items": [
                            #(<#ty as ::magnet_schema::BsonSchema>::bson_schema(),)*
                        ],
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
