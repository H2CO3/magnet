//! For the time being, `BsonSchema` can't be automatically derived for a `union`.

use syn::{ Attribute, DataUnion };
use proc_macro2::TokenStream;
use error::{ Error, Result };

/// Implements `BsonSchema` for a `union`.
pub fn impl_bson_schema_union(_: Vec<Attribute>, _: DataUnion) -> Result<TokenStream> {
    Err(Error::new("`BsonSchema` can't be implemented for unions"))
}
