//! For the time being, `BsonSchema` can't be automatically derived for a `union`.

use quote::Tokens;
use syn::{ Attribute, DataUnion };
use error::{ Error, Result };

/// Implements `BsonSchema` for a `union`.
pub fn impl_bson_schema_union(_: Vec<Attribute>, _: DataUnion) -> Result<Tokens> {
    Err(Error::new("`BsonSchema` can't be implemented for unions"))
}
