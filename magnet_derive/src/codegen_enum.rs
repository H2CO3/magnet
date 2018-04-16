//! Code generation for `enum`s.

use quote::Tokens;
use syn::{ Attribute, DataEnum };
use error::{ Error, Result };

/// Implements `BsonSchema` for an `enum`.
/// TODO(H2CO3): implement me
pub fn impl_bson_schema_enum(_attrs: Vec<Attribute>, _ast: DataEnum) -> Result<Tokens> {
    Err(Error::new("`#[derive(BsonSchema)]` for `enum`s is not implemented"))
}
