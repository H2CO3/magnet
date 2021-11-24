//! # Magnet, a JSON/BSON schema generator
//!
//! These two related crates, `magnet_schema` and `magnet_derive` define
//! a trait, `BsonSchema`, and a proc-macro derive for the same trait,
//! which allows types to easily implement JSON schema validation for
//! use with MongoDB.
//!
//! The trait defines a single function, `bson_schema()`, that returns
//! a BSON `Document` describing the validation schema of the type based
//! on its fields (for `struct`s and tuples), variants (for `enum`s), or
//! elements/entries (for array- and map-like types).
//!
//! The types are expected to be serialized and deserialized using Serde,
//! and generally Magnet will try very hard to respect `#[serde(...)]`
//! annotations as faithfully as possible, but no `Serialize + Deserialize`
//! trait bounds are enforced on the types as this is not strictly necessary.
//!
//! ## Usage Example
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde;
//! #[macro_use]
//! extern crate bson;
//! #[macro_use]
//! extern crate magnet_derive;
//! extern crate magnet_schema;
//!
//! use std::collections::HashSet;
//! use magnet_schema::BsonSchema;
//!
//! #[derive(BsonSchema)]
//! struct Person {
//!     name: String,
//!     nicknames: HashSet<String>,
//!     age: usize,
//!     contact: Option<Contact>,
//! }
//!
//! #[derive(BsonSchema, Serialize, Deserialize)]
//! #[serde(tag = "type", content = "value")]
//! enum Contact {
//!     Email(String),
//!     Phone(u64),
//! }
//!
//! fn main() {
//!     println!("{:#?}", Person::bson_schema());
//! }
//! ```
//!
//! ## Custom Attributes
//!
//! * `#[serde(rename = "new_name")]`: Magnet will respect Serde's field/variant
//!   renaming attribute by default.
//!
//! * `#[serde(rename_all = "rename_rule")]`: it will also respect Serde's
//!   `rename_all` rule.
//!
//! * `#[magnet(min_incl = "-1337")]` &mdash; enforces an inclusive minimum for fields of numeric types
//!
//! * `#[magnet(min_excl = "42")]` &mdash; enforces an exclusive "minimum" (infimum) for fields of numeric types
//!
//! * `#[magnet(max_incl = "63")]` &mdash; enforces an inclusive maximum for fields of numeric types
//!
//! * `#[magnet(max_excl = "64")]` &mdash; enforces an exclusive "maximum" (supremum) for fields of numeric types
//!
//! ## Development Roadmap
//!
//! * `[x]` Define `BsonSchema` trait
//!
//! * `[x]` `impl BsonSchema` for most primitives/`std::` types
//!
//! * `[x]` Cargo `feature`s for implementing `BsonSchema` for "atomic"
//!   types in foreign crates, for instance, `url::Url` and `uuid::Uuid`.
//!
//! * `[x]` `#[derive(BsonSchema)]` on regular, named-field structs
//!
//! * `[x]` `#[derive(BsonSchema)]` on newtype structs
//!
//! * `[x]` `#[derive(BsonSchema)]` on tuple structs
//!
//! * `[x]` `#[derive(BsonSchema)]` on unit structs
//!
//! * `[ ]` `#[derive(BsonSchema)]` on enums
//!
//!   * `[x]` unit variants
//!
//!   * `[ ]` newtype variants
//!
//!     * `[x]` newtype variants around structs and maps
//!
//!     * `[ ]` newtype variants around inner, transitive `enum`s
//!
//!   * `[x]` tuple variants
//!
//!   * `[x]` struct variants
//!
//!   * `[x]` respect Serde tagging conventions: external/internal/adjacent
//!
//! * `[x]` Respect more `#[serde(...)]` attributes, for example: `rename`,
//!   `rename_all`
//!
//! * `[ ]` Respect more `#[serde(...)]` attributes, for example: `default`,
//!   `skip`, `skip_serializing`, `skip_deserializing`
//!
//! * `[x]` Handle generic types in proc-macro derive
//!
//! * `[ ]` Standard (non-MongoDB-specific) JSON schema support (approach?)
//!
//! * `[x]` unit tests
//!
//! * `[x]` documentation for attributes
//!
//! * `[ ]` `impl BsonSchema` for more esoteric primitives/standard types
//!   such as specialization of `[u8]`/`Vec<u8>` as binary, adding a
//!   validation regex `"pattern"` to `Path` and `PathBuf`, etc.
//!
//! * `[ ]` Add our own attributes
//!
//!   * `[x]` `magnet(rename = "...")` &mdash; renames the field or variant
//!     to the name specified as the value of the `rename` attribute
//!
//!   * `[ ]` `magnet(regex = "foo?|[ba]r{3,6}")` &mdash; custom validation;
//!     implies `"type": "string"`. Patterns are implicitly enclosed between
//!     `^...$` for robustness.
//!
//!   * `[ ]` `magnet(unsafe_regex = "^nasty-regex$")` &mdash; just like
//!     `magnet(regex)`, but no automatic enclosing in `^...$` happens.
//!     **This may allow invalid data to pass validation!!!**
//!
//!   * `[ ]` `magnet(non_empty)` &mdash; for collections: same as `min_length = "1"`.
//!
//!   * `[ ]` `magnet(min_length = "16")` &mdash; for collections/tuples etc.
//!
//!   * `[ ]` `magnet(max_length = "32")` &mdash; for collections/tuples etc.
//!
//!   * `[x]` `magnet(min_incl = "-1337")` &mdash; inclusive minimum for numbers
//!
//!   * `[x]` `magnet(min_excl = "42")` &mdash; exclusive "minimum" (infimum) for numbers
//!
//!   * `[x]` `magnet(max_incl = "63")` &mdash; inclusive maximum for numbers
//!
//!   * `[x]` `magnet(max_excl = "64")` &mdash; exclusive "maximum" (supremum) for numbers
//!
//!   * `[ ]` `magnet(allow_extra_fields)` &mdash; sets `"additionalProperties": true`.
//!     By default, Magnet sets this field to `false` for maximal safety.
//!     Allowing arbitrary data to be inserted in a DB is generally a Bad Idea,
//!     as it may lead to code injection (`MongoDB` supports storing JavaScript
//!     in a collection! Madness!) or at best, denial-of-service (DoS) attacks.
//!
//!   * `[ ]` `magnet(allow_extra_fields = "ExtraFieldType")` &mdash; sets
//!     `"additionalProperties": ExtraFieldType::bson_schema()`, so that
//!     unlisted additional object fields are allowed provided that they
//!     conform to the schema of the specified type.

#![doc(html_root_url = "https://docs.rs/magnet_schema/0.9.0")]
#![deny(missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications, missing_docs)]
#![allow(clippy::single_match, clippy::match_same_arms, clippy::match_ref_pats,
         clippy::clone_on_ref_ptr, clippy::needless_pass_by_value)]
#![deny(clippy::wrong_pub_self_convention, clippy::used_underscore_binding,
        clippy::stutter, clippy::similar_names, clippy::pub_enum_variant_names,
        clippy::missing_docs_in_private_items,
        clippy::non_ascii_literal, clippy::unicode_not_nfc,
        clippy::result_unwrap_used, clippy::option_unwrap_used,
        clippy::option_map_unwrap_or_else, clippy::option_map_unwrap_or, clippy::filter_map,
        clippy::shadow_unrelated, clippy::shadow_reuse, clippy::shadow_same,
        clippy::int_plus_one, clippy::string_add_assign, clippy::if_not_else,
        clippy::invalid_upcast_comparisons,
        clippy::cast_precision_loss, clippy::cast_lossless,
        clippy::cast_possible_wrap, clippy::cast_possible_truncation,
        clippy::mutex_integer, clippy::mut_mut, clippy::items_after_statements,
        clippy::print_stdout, clippy::mem_forget, clippy::maybe_infinite_iter)]

#[macro_use]
extern crate bson;
#[cfg(feature = "url")]
extern crate url;
#[cfg(feature = "uuid")]
extern crate uuid;

use std::{ u8, u16, u32, u64, usize, i8, i16, i32, i64, isize };
use std::ffi::{ OsStr, OsString };
use std::path::{ Path, PathBuf };
use std::marker::PhantomData;
use std::hash::{ Hash, BuildHasher };
use std::borrow::Cow;
use std::rc::Rc;
use std::ops::{ Range, RangeInclusive };
use std::cell::{ Cell, RefCell };
use std::sync::{ Arc, Mutex, RwLock };
use std::collections::{
    HashSet, HashMap,
    BTreeSet, BTreeMap,
    VecDeque, BinaryHeap,
    LinkedList,
};
use bson::{ Bson, Document };
use bson::oid::ObjectId;

#[doc(hidden)]
pub mod support;

/// Types which can be expressed/validated by a MongoDB-flavored JSON schema.
pub trait BsonSchema {
    /// Returns a BSON document describing the MongoDB-flavored schema of this type.
    fn bson_schema() -> Document;
}

/////////////////////////////
// Primitive and std types //
/////////////////////////////

impl BsonSchema for bool {
    fn bson_schema() -> Document {
        doc!{ "type": "boolean" }
    }
}

macro_rules! impl_bson_schema_int {
    ($($ty:ident: $min:expr => $max:expr;)*) => {$(
        impl BsonSchema for $ty {
            #[allow(trivial_numeric_casts)]
            #[allow(clippy::cast_possible_wrap, clippy::cast_lossless)]
            fn bson_schema() -> Document {
                doc! {
                    "bsonType": ["int", "long"],
                    "minimum": $min as i64,
                    "maximum": $max as i64,
                }
            }
        }
    )*}
}

impl_bson_schema_int! {
    u8 :  u8::MIN =>  u8::MAX;
    u16: u16::MIN => u16::MAX;
    u32: u32::MIN => u32::MAX;
    u64: u64::MIN => i64::MAX; // !!! must not overflow i64
    i8 :  i8::MIN =>  i8::MAX;
    i16: i16::MIN => i16::MAX;
    i32: i32::MIN => i32::MAX;
    i64: i64::MIN => i64::MAX;
}

#[cfg(any(target_pointer_width =  "8",
          target_pointer_width = "16",
          target_pointer_width = "32"))]
impl BsonSchema for usize {
    fn bson_schema() -> Document {
        doc! {
            "bsonType": ["int", "long"],
            "minimum": usize::MIN as i64,
            "maximum": usize::MAX as i64,
        }
    }
}

/// Do **NOT** assume `sizeof(usize) <= sizeof(u64)`!!!
#[cfg(target_pointer_width = "64")]
impl BsonSchema for usize {
    fn bson_schema() -> Document {
        doc! {
            "bsonType": ["int", "long"],
            "minimum": usize::MIN as i64,
            "maximum": isize::MAX as i64,
        }
    }
}

/// Do **NOT** assume `sizeof(isize) <= sizeof(i64)`!!!
#[cfg(any(target_pointer_width =  "8",
          target_pointer_width = "16",
          target_pointer_width = "32",
          target_pointer_width = "64"))]
impl BsonSchema for isize {
    fn bson_schema() -> Document {
        doc! {
            "bsonType": ["int", "long"],
            "minimum": isize::MIN as i64,
            "maximum": isize::MAX as i64,
        }
    }
}

macro_rules! impl_bson_schema_float {
    ($($ty:ident,)*) => {$(
        impl BsonSchema for $ty {
            fn bson_schema() -> Document {
                doc!{ "type": "number" }
            }
        }
    )*}
}

impl_bson_schema_float!{
    f32,
    f64,
}

macro_rules! impl_bson_schema_string {
    ($($ty:ty,)*) => {$(
        impl BsonSchema for $ty {
            fn bson_schema() -> Document {
                doc!{ "type": "string" }
            }
        }
    )*}
}

// TODO(H2CO3): path-matching regex for `Path` and `PathBuf`?
impl_bson_schema_string! {
    str,
    String,
    OsStr,
    OsString,
    Path,
    PathBuf,
}

///////////////////////////////
// Built-in parametric types //
///////////////////////////////

impl<'a, T> BsonSchema for &'a T where T: ?Sized + BsonSchema {
    fn bson_schema() -> Document {
        T::bson_schema()
    }
}

impl<'a, T> BsonSchema for &'a mut T where T: ?Sized + BsonSchema {
    fn bson_schema() -> Document {
        T::bson_schema()
    }
}

/// TODO(H2CO3): maybe specialize as binary for `[u8]`?
impl<T> BsonSchema for [T] where T: BsonSchema {
    fn bson_schema() -> Document {
        doc! {
            "type": "array",
            "items": T::bson_schema(),
        }
    }
}

macro_rules! impl_bson_schema_array {
    ($($size:expr,)*) => {$(
        impl<T> BsonSchema for [T; $size] where T: BsonSchema {
            #[allow(trivial_numeric_casts)]
            fn bson_schema() -> Document {
                doc! {
                    "type": "array",
                    "minItems": $size as i64,
                    "maxItems": $size as i64,
                    "items": T::bson_schema(),
                }
            }
        }
    )*}
}

impl_bson_schema_array! {
    0,   1,  2,  3,  4,  5,  6,  7,
    8,   9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47,
    48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 58, 59, 60, 61, 62, 63,
}

impl_bson_schema_array! {
    64,     96,  128,  192,  256,   384,   512,   768,
    1024, 1536, 2048, 4096, 8192, 16384, 32768, 65536,
}

impl BsonSchema for () {
    fn bson_schema() -> Document {
        doc! {
            "type": ["array", "null"],
            "maxItems": 0_i64,
        }
    }
}

macro_rules! impl_bson_schema_tuple {
    ($($ty:ident),*) => {
        impl<$($ty),*> BsonSchema for ($($ty),*) where $($ty: BsonSchema),* {
            fn bson_schema() -> Document {
                doc! {
                    "type": "array",
                    "additionalItems": false,
                    "items": [$($ty::bson_schema()),*],
                }
            }
        }
    }
}

impl_bson_schema_tuple!{ A, B }
impl_bson_schema_tuple!{ A, B, C }
impl_bson_schema_tuple!{ A, B, C, D }
impl_bson_schema_tuple!{ A, B, C, D, E }
impl_bson_schema_tuple!{ A, B, C, D, E, F }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I, J }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I, J, K }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I, J, K, L }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I, J, K, L, M }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I, J, K, L, M, N }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O }
impl_bson_schema_tuple!{ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P }

///////////////////////////////////////
// Generics, Containers, Collections //
///////////////////////////////////////

/// TODO(H2CO3): maybe specialize for `Cow<[u8]>` as binary?
impl<'a, T> BsonSchema for Cow<'a, T> where T: ?Sized + Clone + BsonSchema {
    fn bson_schema() -> Document {
        T::bson_schema()
    }
}

impl<T> BsonSchema for Cell<T> where T: BsonSchema {
    fn bson_schema() -> Document {
        T::bson_schema()
    }
}

macro_rules! impl_bson_schema_unsized {
    ($($ty:ident,)*) => {$(
        impl<T> BsonSchema for $ty<T> where T: ?Sized + BsonSchema {
            fn bson_schema() -> Document {
                T::bson_schema()
            }
        }
    )*}
}

impl_bson_schema_unsized! {
    Box,
    Rc,
    Arc,
    RefCell,
    Mutex,
    RwLock,
}

/// TODO(H2CO3): maybe specialize for `Vec<u8>` as binary?
impl<T> BsonSchema for Vec<T> where T: BsonSchema {
    fn bson_schema() -> Document {
        doc! {
            "type": "array",
            "items": T::bson_schema(),
        }
    }
}

impl<T> BsonSchema for VecDeque<T> where T: BsonSchema {
    fn bson_schema() -> Document {
        doc! {
            "type": "array",
            "items": T::bson_schema(),
        }
    }
}

impl<T> BsonSchema for LinkedList<T> where T: BsonSchema {
    fn bson_schema() -> Document {
        doc! {
            "type": "array",
            "items": T::bson_schema(),
        }
    }
}

impl<T> BsonSchema for BinaryHeap<T> where T: BsonSchema + Ord {
    fn bson_schema() -> Document {
        doc! {
            "type": "array",
            "items": T::bson_schema(),
        }
    }
}

impl<T> BsonSchema for Option<T> where T: BsonSchema {
    fn bson_schema() -> Document {
        let mut doc = T::bson_schema();
        let null_bson_str = Bson::from("null");
        let (type_key, old_type_spec) = match doc.remove("type") {
            Some(spec) => ("type", spec),
            None => match doc.remove("bsonType") {
                Some(spec) => ("bsonType", spec),
                None => {
                    // type wasn't directly constrained;
                    // as a last resort, check if it's an `enum`.
                    if let Some(&mut Bson::Array(ref mut array)) = doc.get_mut("anyOf") {
                        array.push(bson!({ "type": null_bson_str }));
                    }
                    return doc;
                }
            }
        };
        let new_type_spec = match old_type_spec {
            Bson::String(_) => vec![
                old_type_spec,
                null_bson_str,
            ],
            Bson::Array(mut array) => {
                // duplicate type strings are a schema error :(
                if !array.iter().any(|item| item == &null_bson_str) {
                    array.push(null_bson_str);
                }

                array
            },
            _ => panic!("invalid schema: `{}` isn't a string or array: {:?}",
                        type_key, old_type_spec.element_type()),
        };

        doc.insert(type_key, new_type_spec);
        doc
    }
}

impl<T, H> BsonSchema for HashSet<T, H>
    where T: BsonSchema + Eq + Hash,
          H: BuildHasher
{
    fn bson_schema() -> Document {
        doc! {
            "type": "array",
            "uniqueItems": true,
            "items": T::bson_schema(),
        }
    }
}

impl<T> BsonSchema for BTreeSet<T> where T: BsonSchema + Ord {
    fn bson_schema() -> Document {
        doc! {
            "type": "array",
            "uniqueItems": true,
            "items": T::bson_schema(),
        }
    }
}

impl<K, V, H> BsonSchema for HashMap<K, V, H>
    where K: ToString + Eq + Hash,
          V: BsonSchema,
          H: BuildHasher
{
    fn bson_schema() -> Document {
        doc! {
            "type": "object",
            "additionalProperties": V::bson_schema(),
        }
    }
}

impl<K, V> BsonSchema for BTreeMap<K, V>
    where K: ToString + Ord,
          V: BsonSchema
{
    fn bson_schema() -> Document {
        doc! {
            "type": "object",
            "additionalProperties": V::bson_schema(),
        }
    }
}

impl<T: BsonSchema> BsonSchema for Range<T> {
    fn bson_schema() -> Document {
        doc! {
            "type": "object",
            "additionalProperties": false,
            "required": ["start", "end"],
            "properties": {
                "start": T::bson_schema(),
                "end":   T::bson_schema(),
            },
        }
    }
}

impl<T: BsonSchema> BsonSchema for RangeInclusive<T> {
    fn bson_schema() -> Document {
        doc! {
            "type": "object",
            "additionalProperties": false,
            "required": ["start", "end"],
            "properties": {
                "start": T::bson_schema(),
                "end":   T::bson_schema(),
            },
        }
    }
}

impl<T> BsonSchema for PhantomData<T> {
    fn bson_schema() -> Document {
        // it's just a unit struct
        <() as BsonSchema>::bson_schema()
    }
}

////////////////////////////////////////////////////////
// Implementations for useful types in foreign crates //
////////////////////////////////////////////////////////

impl BsonSchema for Document {
    fn bson_schema() -> Document {
        doc!{ "type": "object" }
    }
}

impl BsonSchema for ObjectId {
    fn bson_schema() -> Document {
        doc!{ "bsonType": "objectId" }
    }
}

#[cfg(feature = "url")]
impl BsonSchema for url::Url {
    fn bson_schema() -> Document {
        doc! {
            "type": "string",
            // TODO(H2CO3): validation regex pattern?
        }
    }
}

#[cfg(feature = "uuid")]
impl BsonSchema for uuid::Uuid {
    fn bson_schema() -> Document {
        doc! {
            "type": "string",
            "pattern": "^[[:xdigit:]]{8}-[[:xdigit:]]{4}-[[:xdigit:]]{4}-[[:xdigit:]]{4}-[[:xdigit:]]{12}$",
        }
    }
}
