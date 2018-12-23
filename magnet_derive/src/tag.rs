//! Types and functions for dealing with Serde `enum` representation conventions.

use syn::Attribute;
use error::Result;
use meta;

/// Represents Serde's `enum` tagging convention.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::stutter)]
pub enum SerdeEnumTag {
    /// An `enum` with the "externally-tagged" representation. The default.
    /// The map key is the name of the variant.
    External,
    /// An `enum` with the "untagged" representation. There is no map key.
    Untagged,
    /// An `enum` with the "internally-tagged" representation.
    /// The map key is the value of the `tag = "..."` meta.
    Internal(String),
    /// An `enum` with the "adjacently-tagged" representation.
    /// Map keys are the values of the `tag = "...", content = "..."` metas.
    Adjacent {
        /// The key for which the value is the name of the variant.
        tag: String,
        /// The key for which the value is the associated value of the variant.
        content: String,
    },
}

impl SerdeEnumTag {
    /// Attempts to parse some attributes into a Serde enum tagging convention.
    /// TODO(H2CO3): check for conflicting tags? (Serde is supposed to do that!)
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let conv = if let Some(tag) = meta::serde_name_value(attrs, "tag")? {
            let tag = meta::value_as_str(&tag)?;

            if let Some(content) = meta::serde_name_value(attrs, "content")? {
                let content = meta::value_as_str(&content)?;

                SerdeEnumTag::Adjacent { tag, content }
            } else {
                SerdeEnumTag::Internal(tag)
            }
        } else if meta::has_serde_word(attrs, "untagged")? {
            SerdeEnumTag::Untagged
        } else {
            SerdeEnumTag::External
        };

        Ok(conv)
    }
}

impl Default for SerdeEnumTag {
    fn default() -> Self {
        SerdeEnumTag::External
    }
}
