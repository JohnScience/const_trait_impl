mod filter_attrs;

use crate::{
    Comma, GenericParam, Generics, Pair, PredicateType, TildeConst, TokensOrDefault, TraitBound,
    TraitBoundModifier, TypeParam, TypeParamBound, WhereClause, WherePredicate,
};
use filter_attrs::FilterAttrs;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt};
use syn::Token;

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for PredicateType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.lifetimes.to_tokens(tokens);
        self.bounded_ty.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.bounds.to_tokens(tokens);
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for WherePredicate {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            WherePredicate::Type(t) => t.to_tokens(tokens),
            WherePredicate::Lifetime(lt) => lt.to_tokens(tokens),
            WherePredicate::Eq(eq) => eq.to_tokens(tokens),
        }
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for WhereClause {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.where_token.to_tokens(tokens);
        self.predicates.to_tokens(tokens);
    }
}

impl ToTokens for TildeConst {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.tilde.to_tokens(tokens);
        self.const_.to_tokens(tokens);
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for TraitBoundModifier {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            TraitBoundModifier::None => {}
            TraitBoundModifier::Maybe(t) => t.to_tokens(tokens),
            TraitBoundModifier::TildeConst(tilde_const) => tilde_const.to_tokens(tokens),
        }
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for TraitBound {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let to_tokens = |tokens: &mut TokenStream2| {
            self.modifier.to_tokens(tokens);
            self.lifetimes.to_tokens(tokens);
            {
                self.path.to_tokens(tokens);
            }
        };
        match &self.paren_token {
            Some(paren) => paren.surround(tokens, to_tokens),
            None => to_tokens(tokens),
        }
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for TypeParamBound {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            TypeParamBound::Trait(t) => t.to_tokens(tokens),
            TypeParamBound::Lifetime(l) => l.to_tokens(tokens),
        }
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for GenericParam {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            GenericParam::Type(_e) => _e.to_tokens(tokens),
            GenericParam::Lifetime(_e) => _e.to_tokens(tokens),
            GenericParam::Const(_e) => _e.to_tokens(tokens),
        }
    }
}

impl<'a, T> ToTokens for TokensOrDefault<'a, T>
where
    T: ToTokens + Default,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.0 {
            Some(t) => t.to_tokens(tokens),
            None => T::default().to_tokens(tokens),
        }
    }
}

// generics.rs (syn 1.0.86)
impl ToTokens for Generics {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.params.is_empty() {
            return;
        }

        TokensOrDefault(&self.lt_token).to_tokens(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        //
        // TODO: ordering rules for const parameters vs type parameters have
        // not been settled yet. https://github.com/rust-lang/rust/issues/44580
        let mut trailing_or_empty = true;
        for param in self.params.pairs() {
            if let GenericParam::Lifetime(_) = **param.value() {
                <Pair<&GenericParam, &Comma> as ToTokens>::to_tokens(&param, tokens);
                trailing_or_empty = param.punct().is_some();
            }
        }
        for param in self.params.pairs() {
            match **param.value() {
                GenericParam::Type(_) | GenericParam::Const(_) => {
                    if !trailing_or_empty {
                        <Token![,]>::default().to_tokens(tokens);
                        trailing_or_empty = true;
                    }
                    param.to_tokens(tokens);
                }
                GenericParam::Lifetime(_) => {}
            }
        }

        TokensOrDefault(&self.gt_token).to_tokens(tokens);
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl ToTokens for TypeParam {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.append_all(self.attrs.outer());
        self.ident.to_tokens(tokens);
        if !self.bounds.is_empty() {
            TokensOrDefault(&self.colon_token).to_tokens(tokens);
            self.bounds.to_tokens(tokens);
        }
        if let Some(default) = &self.default {
            TokensOrDefault(&self.eq_token).to_tokens(tokens);
            default.to_tokens(tokens);
        }
    }
}
