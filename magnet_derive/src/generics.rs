//! Parse and extend generic bounds.

use syn::{
    Generics, ImplGenerics, TypeGenerics, GenericParam,
    WhereClause, WherePredicate, PredicateType,
    TypeParamBound, TraitBound, TraitBoundModifier, TypePath,
    Ident, Path, PathSegment,
};
use syn::punctuated::Punctuated;
use syn::token::{ Colon2, Add };
use proc_macro2::Span;

/// Helper for extending generics with the `: BsonSchema` trait bound.
#[allow(clippy::stutter)]
pub trait GenericsExt: Sized {
    /// The first return value is the `impl` generic parameter list on the left.
    /// The second one is just the list of names of type and lifetime arguments.
    /// The third one is the augmented `where` clause -- the whole point.
    fn split_and_augment_for_impl(&self) -> (
        ImplGenerics,
        TypeGenerics,
        Option<WhereClause>,
    );
}

impl GenericsExt for Generics {
    fn split_and_augment_for_impl(&self) -> (
        ImplGenerics,
        TypeGenerics,
        Option<WhereClause>,
    ) {
        let (impl_generics, type_generics, where_clause) = self.split_for_impl();
        let mut where_clause = where_clause.cloned().unwrap_or(WhereClause {
            where_token: Default::default(),
            predicates:  Default::default(),
        });

        where_clause.predicates.extend(self.params
                                       .iter()
                                       .filter_map(where_predicate));

        let where_clause = if where_clause.predicates.is_empty() {
            None
        } else {
            Some(where_clause)
        };

        (impl_generics, type_generics, where_clause)
    }
}

/// Returns the `BsonSchema` type bound.
fn bson_schema_type_bounds() -> Punctuated<TypeParamBound, Add> {
    let bound = TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: Path {
            leading_colon: Colon2::default().into(),
            segments: vec![
                PathSegment {
                    ident: Ident::new("magnet_schema", Span::call_site()),
                    arguments: Default::default(),
                },
                PathSegment {
                    ident: Ident::new("BsonSchema", Span::call_site()),
                    arguments: Default::default(),
                },
            ].into_iter().collect(),
        }
    });

    vec![bound].into_iter().collect()
}

/// Returns a predicate for a `where` clause iff the generic param is a type.
fn where_predicate(param: &GenericParam) -> Option<WherePredicate> {
    let type_param = match *param {
        GenericParam::Type(ref ty) => ty,
        _ => return None,
    };

    let bounded_ty = TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: vec![
                PathSegment {
                    ident: type_param.ident.clone(),
                    arguments: Default::default(),
                }
            ].into_iter().collect(),
        },
    };

    let p = WherePredicate::Type(
        PredicateType {
            lifetimes: None,
            bounded_ty: bounded_ty.into(),
            colon_token: Default::default(),
            bounds: bson_schema_type_bounds(),
        }
    );

    Some(p)
}
