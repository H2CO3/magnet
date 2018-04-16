# Magnet, a JSON schema generator

[![Magnet on crates.io](https://img.shields.io/crates/v/magnet_schema.svg)](https://crates.io/crates/magnet_schema)
[![Magnet on docs.rs](https://docs.rs/magnet_schema/badge.svg)](https://docs.rs/magnet_schema)
[![Magnet Download](https://img.shields.io/crates/d/magnet_schema.svg)](https://crates.io/crates/magnet_schema)
[![Magnet License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/H2CO3/magnet/blob/master/LICENSE.txt)
[![Lines of Code](https://tokei.rs/b1/github/H2CO3/magnet)](https://github.com/Aaronepower/tokei)
[![Twitter](https://img.shields.io/badge/twitter-@H2CO3_iOS-blue.svg?style=flat&colorB=64A5DE&label=Twitter)](http://twitter.com/H2CO3_iOS)

These two related crates, `magnet_derive` and `magnet_schema` help you define (and, in most cases, automatically derive) MongoDB-flavored [JSON schemas](https://docs.mongodb.com/manual/reference/operator/query/jsonSchema/#extensions) for your domain model types. Currently, the primary use case for this library is to make it easy to validate serializeable types when using the [MongoDB Rust driver](https://docs.rs/mongodb/).

The defined `BsonSchema` trait defines a single function, `bson_schema`, which should/will return a Bson `Document` that is a valid JSON schema describing the structure of the implementing type. Example:

```rust
#[macro_use]
extern crate bson;
#[macro_use]
extern crate magnet_derive;
extern crate magnet_schema;
extern crate mongodb;

use std::collections::HashSet;
use magnet_schema::BsonSchema;

use mongodb::{ Client, ThreadedClient, Error, Result, CommandType };
use mongodb::db::{ Database, ThreadedDatabase };
use mongodb::coll::Collection;

#[derive(BsonSchema)]
struct Contact {
    name: String,
    nicknames: HashSet<String>,
    age: usize,
    email: Option<String>,
}

fn main() {
    let schema = Contact::bson_schema();
    let spec = doc! {
        "create": "Contact",
        "validator": { "$jsonSchema": schema },
    };
    let client = Client::connect("localhost", 27017).expect("can't connect to mongod");
    let db = client.db("Example");
    let result = db.command(spec, CommandType::CreateCollection, None).expect("network error");
    // etc.
}
```

For milestones and custom `#[attributes]`, please see the [documentation](https://docs.rs/magnet_schema).
