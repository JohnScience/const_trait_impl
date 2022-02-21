use crate::{
    GenericParam, PredicateType, TraitBound, TraitBoundModifier, TypeParam, TypeParamBound,
    WherePredicate, WhereClause, Generics, ItemImpl, ItemConstImpl
};
use syn::{
    punctuated::{Pair, Punctuated},
    token::{Add, Comma},
};

impl From<TraitBoundModifier> for syn::TraitBoundModifier {
    fn from(m: TraitBoundModifier) -> Self {
        match m {
            TraitBoundModifier::None | TraitBoundModifier::TildeConst(_) => {
                syn::TraitBoundModifier::None
            }
            TraitBoundModifier::Maybe(question) => syn::TraitBoundModifier::Maybe(question),
        }
    }
}

impl From<TraitBound> for syn::TraitBound {
    fn from(b: TraitBound) -> Self {
        let TraitBound {
            paren_token,
            modifier,
            lifetimes,
            path,
        } = b;
        Self {
            paren_token,
            modifier: modifier.into(),
            lifetimes,
            path,
        }
    }
}

impl From<TypeParamBound> for syn::TypeParamBound {
    fn from(b: TypeParamBound) -> Self {
        match b {
            TypeParamBound::Lifetime(l) => Self::Lifetime(l),
            TypeParamBound::Trait(t) => Self::Trait(t.into()),
        }
    }
}

impl From<TypeParam> for syn::TypeParam {
    fn from(t: TypeParam) -> Self {
        let TypeParam {
            attrs,
            ident,
            colon_token,
            bounds,
            eq_token,
            default,
        } = t;
        Self {
            attrs,
            ident,
            colon_token,
            bounds: bounds
                .into_pairs()
                .filter_map(|pair| {
                    let drop_bound_filter_map = |b: TypeParamBound| {
                        if let TypeParamBound::Trait(tb) = b {
                            let TraitBound {
                                paren_token,
                                modifier,
                                lifetimes,
                                path,
                            } = tb;
                            match modifier {
                                TraitBoundModifier::TildeConst(tc) => {
                                    if path.segments.last().unwrap().ident.to_string() == "Drop" {
                                        None
                                    } else {
                                        let modifier = TraitBoundModifier::TildeConst(tc);
                                        let tb = TraitBound {
                                            paren_token,
                                            modifier,
                                            lifetimes,
                                            path,
                                        };
                                        Some(TypeParamBound::Trait(tb))
                                    }
                                }
                                _ => {
                                    let tb = TraitBound {
                                        paren_token,
                                        modifier,
                                        lifetimes,
                                        path,
                                    };
                                    Some(TypeParamBound::Trait(tb))
                                }
                            }
                        } else {
                            Some(b)
                        }
                    };
                    match pair {
                        Pair::<TypeParamBound, Add>::Punctuated(b, add) => drop_bound_filter_map(b)
                            .map(|b| Pair::<TypeParamBound, Add>::Punctuated(b, add)),
                        Pair::<TypeParamBound, Add>::End(b) => {
                            drop_bound_filter_map(b).map(Pair::<TypeParamBound, Add>::End)
                        }
                    }
                })
                .map(|pair| match pair {
                    Pair::<TypeParamBound, Add>::Punctuated(b, add) => {
                        Pair::<syn::TypeParamBound, Add>::Punctuated(b.into(), add)
                    }
                    Pair::<TypeParamBound, Add>::End(b) => {
                        Pair::<syn::TypeParamBound, Add>::End(b.into())
                    }
                })
                .collect::<Punctuated<syn::TypeParamBound, Add>>(),
            eq_token,
            default,
        }
    }
}

impl From<PredicateType> for syn::PredicateType {
    fn from(
        PredicateType {
            lifetimes,
            bounded_ty,
            colon_token,
            bounds,
        }: PredicateType,
    ) -> Self {
        Self {
            lifetimes,
            bounded_ty,
            colon_token,
            bounds: bounds
                .into_pairs()
                .filter_map(|pair| {
                    let drop_bound_filter_map = |b: TypeParamBound| {
                        if let TypeParamBound::Trait(tb) = b {
                            let TraitBound {
                                paren_token,
                                modifier,
                                lifetimes,
                                path,
                            } = tb;
                            match modifier {
                                TraitBoundModifier::TildeConst(tc) => {
                                    if path.segments.last().unwrap().ident.to_string() == "Drop" {
                                        None
                                    } else {
                                        let modifier = TraitBoundModifier::TildeConst(tc);
                                        let tb = TraitBound {
                                            paren_token,
                                            modifier,
                                            lifetimes,
                                            path,
                                        };
                                        Some(TypeParamBound::Trait(tb))
                                    }
                                }
                                _ => {
                                    let tb = TraitBound {
                                        paren_token,
                                        modifier,
                                        lifetimes,
                                        path,
                                    };
                                    Some(TypeParamBound::Trait(tb))
                                }
                            }
                        } else {
                            Some(b)
                        }
                    };
                    match pair {
                        Pair::<TypeParamBound, Add>::Punctuated(b, add) => drop_bound_filter_map(b)
                            .map(|b| Pair::<TypeParamBound, Add>::Punctuated(b, add)),
                        Pair::<TypeParamBound, Add>::End(b) => {
                            drop_bound_filter_map(b).map(Pair::<TypeParamBound, Add>::End)
                        }
                    }
                })
                .map(|pair| match pair {
                    Pair::<TypeParamBound, Add>::Punctuated(b, add) => {
                        Pair::<syn::TypeParamBound, Add>::Punctuated(b.into(), add)
                    }
                    Pair::<TypeParamBound, Add>::End(b) => {
                        Pair::<syn::TypeParamBound, Add>::End(b.into())
                    }
                })
                .collect::<Punctuated<syn::TypeParamBound, Add>>(),
        }
    }
}

impl From<GenericParam> for syn::GenericParam {
    fn from(param: GenericParam) -> Self {
        match param {
            GenericParam::Const(c) => syn::GenericParam::Const(c),
            GenericParam::Lifetime(l) => syn::GenericParam::Lifetime(l),
            GenericParam::Type(t) => syn::GenericParam::Type(t.into()),
        }
    }
}

impl From<WherePredicate> for syn::WherePredicate {
    fn from(predicate: WherePredicate) -> Self {
        match predicate {
            WherePredicate::Eq(eq) => syn::WherePredicate::Eq(eq),
            WherePredicate::Lifetime(lt) => syn::WherePredicate::Lifetime(lt),
            WherePredicate::Type(ty) => syn::WherePredicate::Type(ty.into()),
        }
    }
}

impl From<WhereClause> for syn::WhereClause {
    fn from(
        WhereClause {
            where_token,
            predicates,
        }: WhereClause,
    ) -> Self {
        Self {
            where_token,
            predicates: predicates
                .into_pairs()
                .map(|pair| match pair {
                    Pair::<WherePredicate, Comma>::Punctuated(p, comma) => {
                        Pair::<syn::WherePredicate, Comma>::Punctuated(p.into(), comma)
                    }
                    Pair::<WherePredicate, Comma>::End(p) => {
                        Pair::<syn::WherePredicate, Comma>::End(p.into())
                    }
                })
                .collect::<Punctuated<syn::WherePredicate, Comma>>(),
        }
    }
}

impl From<Generics> for syn::Generics {
    fn from(generics: Generics) -> Self {
        let Generics {
            lt_token,
            params,
            gt_token,
            where_clause,
        } = generics;
        // The code below reallocates. Fix it later
        Self {
            lt_token,
            params: params
                .into_pairs()
                .map(|pair| match pair {
                    Pair::<GenericParam, Comma>::Punctuated(p, comma) => {
                        Pair::<syn::GenericParam, Comma>::Punctuated(p.into(), comma)
                    }
                    Pair::<GenericParam, Comma>::End(p) => {
                        Pair::<syn::GenericParam, Comma>::End(p.into())
                    }
                })
                .collect::<Punctuated<syn::GenericParam, Comma>>(),
            gt_token,
            where_clause: where_clause.map(<WhereClause as Into<syn::WhereClause>>::into),
        }
    }
}

impl From<ItemConstImpl> for ItemImpl {
    fn from(item_const_impl: ItemConstImpl) -> Self {
        let ItemConstImpl {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            constness: _constness,
            trait_,
            self_ty,
            brace_token,
            items,
        } = item_const_impl;
        Self {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics: generics.into(),
            trait_,
            self_ty,
            brace_token,
            items,
        }
    }
}

// Currently unused
//impl From<ItemConstImpl> for TokenStream {
//    #[allow(unused_variables, clippy::let_and_return)]
//    fn from(item_impl: ItemConstImpl) -> TokenStream {
//        let ItemConstImpl {
//            attrs,
//            defaultness,
//            unsafety,
//            impl_token,
//            generics,
//            constness,
//            trait_,
//            self_ty,
//            brace_token,
//            items,
//        } = item_impl;
//        let Generics {
//            lt_token,
//            gt_token,
//            params,
//            where_clause,
//        } = generics;
//        let mut ts = TokenStream::new();
//        for attr in attrs.into_iter() {
//            ts.extend::<TokenStream>(attr.to_token_stream().into());
//        }
//        ts.extend::<TokenStream>(defaultness.to_token_stream().into());
//        ts.extend::<TokenStream>(unsafety.to_token_stream().into());
//        ts.extend::<TokenStream>(impl_token.to_token_stream().into());
//        ts.extend::<TokenStream>(lt_token.to_token_stream().into());
//        ts.extend::<TokenStream>(params.to_token_stream().into());
//        ts.extend::<TokenStream>(gt_token.to_token_stream().into());
//        ts.extend::<TokenStream>(constness.to_token_stream().into());
//        match trait_ {
//            None => {}
//            Some((bang, path, for_)) => {
//                ts.extend::<TokenStream>(bang.to_token_stream().into());
//                ts.extend::<TokenStream>(path.to_token_stream().into());
//                ts.extend::<TokenStream>(for_.to_token_stream().into());
//            }
//        };
//        ts.extend::<TokenStream>(self_ty.to_token_stream().into());
//        ts.extend::<TokenStream>(where_clause.to_token_stream().into());
//        let mut nested_ts = TokenStream2::new();
//        for item in items.into_iter() {
//            nested_ts.extend(item.to_token_stream());
//        }
//        ts.extend::<TokenStream>(
//            Group2::new(Delimiter2::Brace, nested_ts)
//                .to_token_stream()
//                .into(),
//        );
//        ts
//    }
//}
