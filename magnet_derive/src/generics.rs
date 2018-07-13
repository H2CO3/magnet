//! Parse and extend generic bounds.

use syn::{ Generics, GenericParam, WhereClause, WherePredicate, PredicateType };
use syn::{ TypeParamBound, TraitBound, TraitBoundModifier, TypePath };
use syn::{ Ident, Path, PathSegment };
use syn::punctuated::Punctuated;
use syn::token::{ Colon2, Add };
use proc_macro2::{ TokenStream, Span };
use quote::ToTokens;

/// Helper for extending generics with the `: BsonSchema` trait bound.
pub trait GenericsExt: Sized {
    /// The first return value is the `impl` generic parameter list on the left.
    /// The second one is just the list of names of type and lifetime arguments.
    /// The third one is the augmented `where` clause -- the whole point.
    fn with_bson_schema(self) -> (TokenStream, TokenStream, TokenStream);
}

impl GenericsExt for Generics {
    fn with_bson_schema(self) -> (TokenStream, TokenStream, TokenStream) {
        // no type parameters
        if self.lt_token.is_none() || self.gt_token.is_none() {
            return (TokenStream::new(), TokenStream::new(), TokenStream::new());
        }

        let self_params: Vec<_> = self.params
            .iter()
            .cloned()
            .map(|param| match param {
                GenericParam::Type(ty) => ty.ident.into_token_stream(),
                GenericParam::Lifetime(lt) => lt.lifetime.into_token_stream(),
                GenericParam::Const(cst) => cst.ident.into_token_stream(),
            })
            .collect();

        let mut where_clause = self.where_clause.unwrap_or(WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });

        where_clause.predicates.extend(self.params
                                       .iter()
                                       .filter_map(where_predicate));

        (
            self.params.into_token_stream(),
            quote!{ #(#self_params),* },
            where_clause.into_token_stream(),
        )
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
