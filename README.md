# Magnet, a JSON schema generator

[![Magnet on crates.io](https://img.shields.io/crates/v/magnet_schema.svg)](https://crates.io/crates/magnet_schema)
[![Magnet on docs.rs](https://docs.rs/magnet_schema/badge.svg)](https://docs.rs/magnet_schema)
[![Magnet Download](https://img.shields.io/crates/d/magnet_schema.svg)](https://crates.io/crates/magnet_schema)
[![Magnet License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/H2CO3/magnet/blob/master/LICENSE.txt)
[![Lines of Code](https://tokei.rs/b1/github/H2CO3/magnet)](https://github.com/Aaronepower/tokei)
[![Twitter](https://img.shields.io/badge/twitter-@H2CO3_iOS-blue.svg?style=flat&colorB=64A5DE&label=Twitter)](http://twitter.com/H2CO3_iOS)

These two related crates, `magnet_derive` and `magnet_schema` help you define (and, in most cases, automatically derive) MongoDB-flavored [JSON schemas](https://docs.mongodb.com/manual/reference/operator/query/jsonSchema/#extensions) for your domain model types. Currently, the primary use case for this library is to make it easy to validate serializeable types when using [Avocado](https://docs.rs/avocado/) or the [MongoDB Rust driver](https://docs.rs/mongodb/).

The defined `BsonSchema` trait defines a single function, `bson_schema`, which should/will return a Bson `Document` that is a valid JSON schema describing the structure of the implementing type. Example:

```rust
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate bson;
#[macro_use]
extern crate magnet_derive;
extern crate magnet_schema;
extern crate mongodb;

use std::collections::HashSet;
use magnet_schema::BsonSchema;

use mongodb::{ Client, ThreadedClient, CommandType };
use mongodb::db::{ ThreadedDatabase };

#[derive(BsonSchema)]
struct Person {
    name: String,
    nicknames: HashSet<String>,
    age: usize,
    contact: Option<Contact>,
}

#[derive(BsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum Contact {
    Email(String),
    Phone(u64),
}

fn main() {
    let schema = Person::bson_schema();
    let spec = doc! {
        "create": "Person",
        "validator": { "$jsonSchema": schema },
    };
    let client = Client::connect("localhost", 27017).expect("can't connect to mongod");
    let db = client.db("Example");
    db.command(spec, CommandType::CreateCollection, None).expect("network error");
    // etc.
}
```

For milestones and custom `#[attributes]`, please see the [documentation](https://docs.rs/magnet_schema).

## Release Notes

### v0.7.0

* Upgrade `uuid` dependency to 0.7.1, and include `v4` and `serde` features
* Upgrade `url` dependency to `1.7.2`

### v0.6.0

* `impl BsonSchema` for arrays of size 2<sup>N</sup> between 128 and 65536; and sizes 1.5 * 2<sup>N</sup> between 96 and 1536.
* Rewrite generics handling using `syn::Generics::split_for_impl`
* Use scoped lints in `magnet_schema` as well

### v0.5.0

* Handle generic types with default generic parameters correctly, by not including the defaults in the generated `impl` (which would result in a compiler error)
* Use scoped lints for Clippy
* Update some dependencies

### v0.4.0

* Update `bson` to `0.13.0` and require its `u2i` feature. This version fixes a
  bug where unit struct were serialized as 1-element arrays. The `u2i` feature
  allows unsigned integers (within the appropriate range) to be serialized as
  signed integers.

### v0.3.3

* Fix a bug where `Option<enum>` was not allowed to be `null`/`None` by the
  generated BSON schema
* Remove an incorrect item from the documentation
* Fix several Clippy lints
* Update dependencies

### v0.3.2

* `impl BsonSchema for Document`
* `impl BsonSchema for ObjectId`
* Documentation improvements
* Update dependencies

### v0.3.1

* Relax `Display` bound for `HashMap`/`BTreeMap` keys, use `ToString` instead
* Update `proc_macro2` dependency so that we can use `TokenStream::default()`

### v0.3.0

* Remove `#[magnet(rename = "...")]` attribute
* `UnorderedDoc::eq()`, `assert_doc_eq!` and `assert_doc_ne!` no longer clone their arguments
* Update `syn` and `quote` dependencies
* Improve documentation

### v0.2.1

* Update `bson` dependency

### v0.2.0

* Support for generic types

### v0.1.4

* Unit tests and a test suite have been added.
* Bug fix: `Option::bson_schema()` didn't handle the `bsonType` field, so `Option<integer>` wasn't allowed to be `null`. This has been corrected.
* Bug fix: every generated schema now uses `Bson::I64` for representing array lengths / collection counts
* Enhancement: `impl BsonSchema for { HashMap, BTreeMap }` now has a less stringent trait bound on the key. It is now `Display` instead of `AsRef<str>`.

### v0.1.3

* Add support for `#[magnet(min_incl = "...", min_excl = "...", max_incl = "...", max_excl = "...")]` attributes on struct fields (named as well as newtype and tuple)

### v0.1.2

* Add support for `enum`s, respecting Serde's tagging conventions (untagged/external/internal/adjacent), except newtype variants around other (inner) `enum`s
* Refactoring, code quality improvements

### v0.1.1

* Add support for newtype structs and tuple structs
* Respect `#[serde(rename_all = "...")]` and `#[serde(rename = "...")]` attributes
* Add Serde-conform case conversion
* Code formatting / organization and documentation improvements

### v0.1.0

* Initial release, only regular structs with named fields are supported
