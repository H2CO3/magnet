#![recursion_limit = "128"]

#[macro_use]
extern crate bson;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate magnet_derive;
extern crate magnet_schema;
extern crate serde_json;

use std::io;
use std::fmt;
use std::str;
use std::error;
use std::cmp::PartialEq;
use magnet_schema::BsonSchema;
use bson::{ Bson, Document };

/// An unordered document: one that doesn't care about the order of its keys.
#[derive(Debug, Clone)]
struct UnorderedDoc(Document);

impl PartialEq for UnorderedDoc {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.iter().all(|(key, value_lhs)| {
            let value_rhs = match other.0.get(key) {
                None => return false,
                Some(bson) => bson,
            };

            match (value_lhs, value_rhs) {
                (&Bson::Document(ref doc_lhs),
                 &Bson::Document(ref doc_rhs)) => {
                    let unord_lhs = UnorderedDoc(doc_lhs.clone());
                    let unord_rhs = UnorderedDoc(doc_rhs.clone());
                    unord_lhs == unord_rhs
                },
                (&Bson::Array(ref arr_lhs), &Bson::Array(ref arr_rhs)) => {
                    if arr_lhs.len() != arr_rhs.len() {
                        return false;
                    }

                    arr_lhs.iter().zip(arr_rhs).all(|args| match args {
                        (&Bson::Document(ref doc_lhs),
                         &Bson::Document(ref doc_rhs)) => {
                            let unord_lhs = UnorderedDoc(doc_lhs.clone());
                            let unord_rhs = UnorderedDoc(doc_rhs.clone());
                            unord_lhs == unord_rhs
                        },
                        _ => args.0 == args.1,
                    })
                },
                _ => value_lhs == value_rhs,
            }
        })
    }
}

impl fmt::Display for UnorderedDoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            serde_json::to_writer_pretty(
                FmtIoWriter(f),
                &self.0
            ).map_err(
                |_| fmt::Error
            )
        } else {
            self.0.fmt(f) // output is prettier than serde_json::to_writer()'s
        }
    }
}

/// Wraps an `fmt::Formatter` and implements `io::Write` for it.
struct FmtIoWriter<'a, 'b: 'a>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> io::Write for FmtIoWriter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        fn io_err<E>(error: E) -> io::Error
            where E: Into<Box<error::Error + Send + Sync>> {

            io::Error::new(io::ErrorKind::Other, error)
        }

        let s = str::from_utf8(buf).map_err(io_err)?;
        self.0.write_str(s).map_err(io_err)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

macro_rules! assert_doc_eq {
    ($lhs:expr, $rhs:expr) => ({
        let lhs_str = stringify!($lhs);
        let rhs_str = stringify!($rhs);

        let lhs = UnorderedDoc($lhs.clone());
        let rhs = UnorderedDoc($rhs.clone());

        assert!(lhs == rhs,
                "Line: {}, {} != {}! Values:\n{:#}\n-- VS. --\n{:#}",
                line!(), lhs_str, rhs_str, lhs, rhs);
    })
}

macro_rules! assert_doc_ne {
    ($lhs:expr, $rhs:expr) => ({
        let lhs_str = stringify!($lhs);
        let rhs_str = stringify!($rhs);

        let lhs = UnorderedDoc($lhs.clone());
        let rhs = UnorderedDoc($rhs.clone());

        assert!(lhs != rhs,
                "Line {}: {} == {}! Values:\n{:#}\n-- VS. --\n{:#}",
                line!(), lhs_str, rhs_str, lhs, rhs);
    })
}

#[test]
fn unordered_doc_equality() {
    let d1 = doc! {
        "foo": "bar",
        "qux": 42,
        "key": [
            {
                "inner_1": null,
                "inner_2": 1337,
            },
            {
                "inner_3": "value",
                "inner_4": -42,
            },
        ],
        "inner": {
            "one": false,
            "other": true,
        },
    };

    let d2 = doc! {
        "key": [
            {
                "inner_2": 1337,
                "inner_1": null,
            },
            {
                "inner_4": -42,
                "inner_3": "value",
            },
        ],
        "foo": "bar",
        "qux": 42,
        "inner": {
            "other": true,
            "one": false,
        },
    };

    let d3 = doc! {
        "key": [
            {
                "inner_3": "value",
                "inner_4": -42,
            },
            {
                "inner_1": null,
                "inner_2": 1337,
            },
        ],
        "foo": "bar",
        "qux": 42,
        "inner": {
            "other": true,
            "one": false,
        },
    };

    assert_doc_eq!(d1, d2);
    assert_doc_eq!(d2, d1);

    assert_doc_ne!(d1, d3);
    assert_doc_ne!(d3, d1);

    assert_doc_ne!(d2, d3);
    assert_doc_ne!(d3, d2);
}

#[test]
fn unit_struct() {
    #[derive(BsonSchema)]
    struct FstUnit;

    #[derive(BsonSchema)]
    struct SndUnit();

    let unit_schema = doc! {
        "type": ["array", "null"],
        "maxItems": 0_i64,
    };

    let fst_schema = FstUnit::bson_schema();
    let snd_schema = SndUnit::bson_schema();

    assert_doc_eq!(fst_schema, snd_schema);
    assert_doc_eq!(snd_schema, fst_schema);

    assert_doc_eq!(fst_schema, unit_schema);
    assert_doc_eq!(snd_schema, unit_schema);
}

#[test]
fn newtype_struct() {
    #[derive(BsonSchema)]
    struct FloatingPoint(f64);

    #[derive(BsonSchema)]
    struct Angle(
        #[magnet(min_incl = "-180", max_excl = "180")]
        f32
    );

    assert_doc_eq!(FloatingPoint::bson_schema(), f64::bson_schema());

    assert_doc_eq!(Angle::bson_schema(), doc! {
        "type": "number",
        "minimum": -180.0,
        "exclusiveMinimum": false,
        "maximum": 180.0,
        "exclusiveMaximum": true,
    });
}

#[test]
fn tuple_struct() {
    #[derive(BsonSchema)]
    struct Complex(f64, f64);

    #[derive(BsonSchema)]
    struct IntRange(Option<u32>, Option<u32>);

    assert_doc_eq!(Complex::bson_schema(), doc! {
        "type": "array",
        "additionalItems": false,
        "items": [
            { "type": "number" },
            { "type": "number" },
        ],
    });

    assert_doc_eq!(IntRange::bson_schema(), doc! {
        "type": "array",
        "additionalItems": false,
        "items": [
            {
                "minimum": std::u32::MIN as i64,
                "maximum": std::u32::MAX as i64,
                "bsonType": ["int", "long", "null"],
            },
            {
                "minimum": std::u32::MIN as i64,
                "maximum": std::u32::MAX as i64,
                "bsonType": ["int", "long", "null"],
            },
        ],
    });
}

#[test]
fn struct_with_named_fields() {
    use std::collections::BTreeMap;

    #[derive(BsonSchema)]
    struct Contact {
        names: Vec<String>,
        address_lines: [String; 3],
        phone_no: Option<u64>,
        email: Option<Email>,
        misc_info: Option<BTreeMap<String, String>>,
    }

    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
    struct Email {
        #[serde(rename = "aDdReSs")]
        address: String,
        provider_name: String,
    }

    assert_doc_eq!(Contact::bson_schema(), doc! {
        "type": "object",
        "additionalProperties": false,
        "required": [
            "names",
            "address_lines",
            "phone_no",
            "email",
            "misc_info",
        ],
        "properties": {
            "names": {
                "type": "array",
                "items": {
                    "type": "string",
                }
            },
            "address_lines": {
                "type": "array",
                "items": {
                    "type": "string",
                },
                "minItems": 3 as i64,
                "maxItems": 3 as i64,
            },
            "phone_no": {
                "bsonType": ["int", "long", "null"],
                "minimum": std::u64::MIN as i64,
                "maximum": std::i64::MAX,
            },
            "email": {
                "type": ["object", "null"],
                "additionalProperties": false,
                "required": ["aDdReSs", "PROVIDER-NAME"],
                "properties": {
                    "aDdReSs": { "type": "string" },
                    "PROVIDER-NAME": { "type": "string" },
                },
            },
            "misc_info": {
                "type": ["object", "null"],
                "additionalProperties": {
                    "type": "string",
                },
            },
        },
    });
}

#[test]
fn untagged_enum() {
    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(untagged)]
    enum Untagged {
        Unit,
        NewType(Option<String>),
        TwoTuple(u8, i16),
        Struct {
            field: i32,
        },
    }

    assert_doc_eq!(Untagged::bson_schema(), doc! {
        "anyOf": [
            {
                "type": ["array", "null"],
                "maxItems": 0_i64,
            },
            {
                "type": ["string", "null"],
            },
            {
                "type": "array",
                "additionalItems": false,
                "items": [
                    {
                        "bsonType": ["int", "long"],
                        "minimum": std::u8::MIN as i64,
                        "maximum": std::u8::MAX as i64,
                    },
                    {
                        "bsonType": ["int", "long"],
                        "minimum": std::i16::MIN as i64,
                        "maximum": std::i16::MAX as i64,
                    },
                ],
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": [ "field" ],
                "properties": {
                    "field": {
                        "bsonType": ["int", "long"],
                        "minimum": std::i32::MIN as i64,
                        "maximum": std::i32::MAX as i64,
                    },
                },
            },
        ]
    });
}

#[test]
fn externally_tagged_enum() {
    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(rename_all = "snake_case")]
    enum ExternallyTagged {
        Unit,
        NewType(Option<String>),
        TwoTuple(u8, i16),
        Struct {
            field: i32,
        },
    }

    assert_doc_eq!(ExternallyTagged::bson_schema(), doc! {
        "anyOf": [
            {
                "enum": ["unit"],
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": [ "new_type" ],
                "properties": {
                    "new_type": {
                        "type": ["string", "null"],
                    },
                },
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["two_tuple"],
                "properties": {
                    "two_tuple": {
                        "type": "array",
                        "additionalItems": false,
                        "items": [
                            {
                                "bsonType": ["int", "long"],
                                "minimum": std::u8::MIN as i64,
                                "maximum": std::u8::MAX as i64,
                            },
                            {
                                "bsonType": ["int", "long"],
                                "minimum": std::i16::MIN as i64,
                                "maximum": std::i16::MAX as i64,
                            },
                        ],
                    },
                },
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["struct"],
                "properties": {
                    "struct": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["field"],
                        "properties": {
                            "field": {
                                "bsonType": ["int", "long"],
                                "minimum": std::i32::MIN as i64,
                                "maximum": std::i32::MAX as i64,
                            },
                        },
                    },
                },
            },
        ]
    });
}

#[test]
fn adjacently_tagged_enum() {
    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(rename_all = "snake_case", tag = "variant", content = "value")]
    enum AdjacentlyTagged {
        Unit,
        NewType(Option<String>),
        TwoTuple(u8, i16),
        Struct {
            field: i32,
        },
    }

    assert_doc_eq!(AdjacentlyTagged::bson_schema(), doc! {
        "anyOf": [
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["variant"],
                "properties": {
                    "variant": { "enum": ["unit"] },
                },
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["variant", "value"],
                "properties": {
                    "variant": { "enum": ["new_type"] },
                    "value": { "type": ["string", "null"] },
                },
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["variant", "value"],
                "properties": {
                    "variant": { "enum": ["two_tuple"] },
                    "value": {
                        "type": "array",
                        "additionalItems": false,
                        "items": [
                            {
                                "bsonType": ["int", "long"],
                                "minimum": std::u8::MIN as i64,
                                "maximum": std::u8::MAX as i64,
                            },
                            {
                                "bsonType": ["int", "long"],
                                "minimum": std::i16::MIN as i64,
                                "maximum": std::i16::MAX as i64,
                            },
                        ],
                    },
                },
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["variant", "value"],
                "properties": {
                    "variant": { "enum": ["struct"] },
                    "value": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["field"],
                        "properties": {
                            "field": {
                                "bsonType": ["int", "long"],
                                "minimum": std::i32::MIN as i64,
                                "maximum": std::i32::MAX as i64,
                            },
                        },
                    },
                },
            },
        ]
    });
}

#[test]
fn internally_tagged_enum() {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(rename_all = "snake_case", tag = "variant")]
    enum InternallyTagged {
        Unit,
        NewTypeOne(NewType),
        NewTypeTwo(HashMap<String, bool>),
        Struct {
            field: i32,
        },
    }

    #[derive(Serialize, Deserialize, BsonSchema)]
    struct NewType {
        name: String,
    }

    assert_doc_eq!(InternallyTagged::bson_schema(), doc! {
        "anyOf": [
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["variant"],
                "properties": {
                    "variant": { "enum": ["unit"] },
                },
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["name", "variant"],
                "properties": {
                    "variant": { "enum": [ "new_type_one" ] },
                    "name": { "type": "string" },
                },
            },
            {
                "type": "object",
                "required": ["variant"],
                "properties": {
                    "variant": { "enum": [ "new_type_two" ] },
                },
                "additionalProperties": {
                    "type": "boolean",
                },
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["variant", "field"],
                "properties": {
                    "variant": { "enum": [ "struct" ] },
                    "field": {
                        "bsonType": ["int", "long"],
                        "minimum": std::i32::MIN as i64,
                        "maximum": std::i32::MAX as i64,
                    },
                },
            },
        ]
    });
}

#[test]
#[should_panic]
fn malformed_internally_tagged_enum_1() {
    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(tag = "variant")]
    enum Foo {
        Bar(Lol),
    }

    #[derive(Serialize, Deserialize, BsonSchema)]
    struct Lol;

    Foo::bson_schema();
}

#[test]
#[should_panic]
fn malformed_internally_tagged_enum_2() {
    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(tag = "variant")]
    enum Foo {
        Bar(u32),
    }

    Foo::bson_schema();
}

#[test]
#[should_panic]
fn malformed_internally_tagged_enum_3() {
    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(tag = "variant")]
    enum Foo {
        Bar(Option<S>),
    }

    #[derive(Serialize, Deserialize, BsonSchema)]
    struct S {
        f: bool,
    }

    Foo::bson_schema();
}

#[test]
#[should_panic]
fn malformed_internally_tagged_enum_4() {
    #[derive(Serialize, Deserialize, BsonSchema)]
    #[serde(tag = "variant")]
    enum Foo {
        Bar(E),
    }

    #[derive(Serialize, Deserialize, BsonSchema)]
    enum E {
        Qux,
        Moo,
    }

    Foo::bson_schema();
}
