# Magnet, a JSON schema generator

These two related crates, `magnet_derive` and `magnet_schema` help you define (and, in most cases, automatically derive) JSON schemas for your domain model types. Currently, the primary use case for this library is to make it easy to validate serializeable types when using the [MongoDB Rust driver](https://docs.rs/mongodb/).

The defined `BsonSchema` trait defines a single function, `bson_schema`, which should/will return a Bson `Document` that is a valid JSON schema describing the structure of the implementing type. Example:

```rust
#[macro_use]
extern crate bson;
#[macro_use]
extern crate magnet_derive;
extern crate magnet_schema;
extern crate mongodb;

use std::collections::HashMap;
use magnet_schema::BsonSchema;

use mongodb::{ Client, ThreadedClient, Error, Result, CommandType };
use mongodb::db::{ Database, ThreadedDatabase };
use mongodb::coll::Collection;

#[derive(BsonSchema)]
struct Animal {
    age_months: usize,
    species_name: &'static str,
    subspecies_endangered: HashMap<String, bool>,
}

fn main() {
    let schema = Animal::bson_schema();
    let spec = doc! {
        "create": "Animal",
        "validator": { "$jsonSchema": schema },
    };
    let client = connect("localhost", 27017).expect("can't connect to mongod");
    let db = client.db("Example");
    let result = db.command(spec, CommandType::CreateCollection, None).expect("network error");
    // etc.
}
```

For milestones, please see the [documentation](https://docs.rs/magnet_schema).
