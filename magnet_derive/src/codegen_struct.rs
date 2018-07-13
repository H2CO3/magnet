//! Code generation for `struct`s.

use syn::{ DataStruct, Attribute };
use proc_macro2::TokenStream;
use error::Result;
use codegen_field::impl_bson_schema_fields;

/// Implements `BsonSchema` for a `struct`.
pub fn impl_bson_schema_struct(attrs: Vec<Attribute>, ast: DataStruct) -> Result<TokenStream> {
    impl_bson_schema_fields(&attrs, ast.fields)
}
