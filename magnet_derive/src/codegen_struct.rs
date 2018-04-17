//! Code generation for `struct`s.

use quote::Tokens;
use syn::{ DataStruct, Attribute };
use error::Result;
use codegen_field::impl_bson_schema_fields;

/// Implements `BsonSchema` for a `struct`.
pub fn impl_bson_schema_struct(attrs: Vec<Attribute>, ast: DataStruct) -> Result<Tokens> {
    impl_bson_schema_fields(&attrs, ast.fields)
}
