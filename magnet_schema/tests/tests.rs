#[macro_use]
extern crate bson;
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
                (&Bson::Document(ref doc_lhs), &Bson::Document(ref doc_rhs)) => {
                    let unord_lhs = UnorderedDoc(doc_lhs.clone());
                    let unord_rhs = UnorderedDoc(doc_rhs.clone());
                    unord_lhs == unord_rhs
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

macro_rules! unord_doc {
    ($($tokens:tt)*) => {
        UnorderedDoc(doc!{ $($tokens)* })
    }
}

macro_rules! assert_doc_eq {
    ($lhs:expr, $rhs:expr) => ({
        let lhs_str = stringify!($lhs);
        let rhs_str = stringify!($rhs);

        let lhs = &$lhs;
        let rhs = &$rhs;

        assert!(lhs == rhs,
                "{} != {}!!! Values:\n{:#}\n-- VS. --\n{:#}",
                lhs_str, rhs_str, lhs, rhs);
    })
}

macro_rules! assert_doc_ne {
    ($lhs:expr, $rhs:expr) => ({
        let lhs_str = stringify!($lhs);
        let rhs_str = stringify!($rhs);

        let lhs = &$lhs;
        let rhs = &$rhs;

        assert!(lhs != rhs,
                "Line {}: {} == {}! Values:\n{:#}\n-- VS. --\n{:#}",
                line!(), lhs_str, rhs_str, lhs, rhs);
    })
}

#[test]
fn unordered_doc_equality() {
    let d1 = unord_doc! {
        "foo": "bar",
        "qux": 42,
        "key": [
            null,
            1337,
        ],
        "inner": {
            "one": false,
            "other": true,
        },
    };

    let d2 = unord_doc! {
        "key": [
            null,
            1337,
        ],
        "foo": "bar",
        "qux": 42,
        "inner": {
            "other": true,
            "one": false,
        },
    };

    let d3 = unord_doc! {
        "key": [
            1337,
            null,
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
