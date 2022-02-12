use proc_macro::TokenStream;
use proc_macro2::{
    Delimiter as Delimiter2, Group as Group2, Span as Span2, TokenStream as TokenStream2,
};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    braced, bracketed,
    ext::IdentExt,
    parse::{Parse, ParseBuffer, ParseStream},
    parse_macro_input,
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    token::{Bang, Brace, Comma, Const, Default as DefaultKW, For, Gt, Impl, Lt, Pound, Unsafe, Paren},
    AttrStyle, Attribute, ConstParam, Error, Ident, ImplItem, Lifetime, LifetimeDef, Path, Result,
    Token, Type, TypePath, WhereClause, parenthesized, BoundLifetimes, ParenthesizedGenericArguments, PathArguments,
};
// syn::Generics is not suitable for support of const_trait_impl and const_fn_trait_bound
// due to the transitive chain:
//
use syn::Generics;
// use syn::GenericParam;
// use syn::TypeParam;
// use syn::TypeParamBound;
// use syn::TraitBound;
// use syn::TraitBoundModifier;

struct ItemConstImpl {
    attrs: Vec<Attribute>,
    // https://github.com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md
    defaultness: Option<DefaultKW>,
    unsafety: Option<Unsafe>,
    impl_token: Impl,
    generics: Generics,
    constness: Option<Const>,
    trait_: Option<(Option<Bang>, Path, For)>,
    self_ty: Box<Type>,
    brace_token: Brace,
    items: Vec<ImplItem>,
}

// // generics.rs (syn 1.0.86)
// #[derive(Default)]
// struct Generics {
//     lt_token: Option<Lt>,
//     params: Punctuated<GenericParam, Comma>,
//     gt_token: Option<Gt>,
//     where_clause: Option<WhereClause>,
// }

// // generics.rs (syn 1.0.86)
// enum GenericParam {
//     /// A generic type parameter: `T: Into<String>`.
//     Type(TypeParam),

//     /// A lifetime definition: `'a: 'b + 'c + 'd`.
//     Lifetime(LifetimeDef),

//     /// A const generic parameter: `const LENGTH: usize`.
//     Const(ConstParam),
// }

// // generics.rs (syn 1.0.86)
// struct TypeParam {
//     pub attrs: Vec<Attribute>,
//     pub ident: Ident,
//     pub colon_token: Option<Token![:]>,
//     pub bounds: Punctuated<TypeParamBound, Token![+]>,
//     pub eq_token: Option<Token![=]>,
//     pub default: Option<Type>,
// }

// // generics.rs (syn 1.0.86)
// enum TypeParamBound {
//     Trait(TraitBound),
//     Lifetime(Lifetime),
// }

// // generics.rs (syn 1.0.86)
// struct TraitBound {
//     pub paren_token: Option<Paren>,
//     pub modifier: TraitBoundModifier,
//     /// The `for<'a>` in `for<'a> Foo<&'a T>`
//     pub lifetimes: Option<BoundLifetimes>,
//     /// The `Foo<&'a T>` in `for<'a> Foo<&'a T>`
//     pub path: Path,
// }

// // generics.rs (syn 1.0.86)
// enum TraitBoundModifier {
//     None,
//     Maybe(Token![?]),
//     TildeConst(TildeConst),
// }

// struct TildeConst {
//     tilde: Token![~],
//     const_: Token![const],
// }

// impl ToTokens for TildeConst {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         self.tilde.to_tokens(tokens);
//         self.const_.to_tokens(tokens);
//     }
// }

// impl Parse for TildeConst {
//     fn parse(input: ParseStream) -> Result<Self> {
//         Ok(Self {
//             tilde: input.parse::<Token![~]>()?,
//             const_: input.parse::<Token![const]>()?,
//         })
//     }
// }

// // generics.rs (syn 1.0.86)
// impl Parse for TraitBoundModifier {
//     fn parse(input: ParseStream) -> Result<Self> {
//         if input.peek(Token![?]) {
//             input.parse().map(TraitBoundModifier::Maybe)
//         } else if input.peek(Token![~]) && input.peek2(Token![const]) {
//             input.parse().map(TraitBoundModifier::TildeConst)
//         } else {
//             Ok(TraitBoundModifier::None)
//         }
//     }
// }

// // generics.rs (syn 1.0.86)
// // Originally, the code was generated with a macro
// impl ToTokens for TraitBoundModifier {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         match self {
//             TraitBoundModifier::None => {},
//             TraitBoundModifier::Maybe(t) => t.to_tokens(tokens),
//             TraitBoundModifier::TildeConst(tilde_const) => tilde_const.to_tokens(tokens),
//         }
//     }
// }

// // generics.rs (syn 1.0.86)
// // Originally, the code was generated with a macro
// impl ToTokens for TraitBound {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         let to_tokens = |tokens: &mut TokenStream2| {
//             self.modifier.to_tokens(tokens);
//             self.lifetimes.to_tokens(tokens);
//             {
//                 self.path.to_tokens(tokens);
//             }
//         };
//         match &self.paren_token {
//             Some(paren) => paren.surround(tokens, to_tokens),
//             None => to_tokens(tokens),
//         }
//     }
// }

// // generics.rs (syn 1.0.86)
// impl Parse for TraitBound {
//     fn parse(input: ParseStream) -> Result<Self> {
//         // The code was originally uncommented 
//         //
//         // let tilde_const = if input.peek(Token![~]) && input.peek2(Token![const]) {
//         //     let tilde_token = input.parse::<Token![~]>()?;
//         //     let const_token = input.parse::<Token![const]>()?;
//         //     Some((tilde_token, const_token))
//         // } else {
//         //     None
//         // };

//         let modifier: TraitBoundModifier = input.parse()?;
//         let lifetimes: Option<BoundLifetimes> = input.parse()?;

//         let mut path: Path = input.parse()?;
//         if path.segments.last().unwrap().arguments.is_empty()

//         && (input.peek(Paren) || input.peek(Token![::]) && input.peek3(Paren))
//         {
//             input.parse::<Option<Token![::]>>()?;
//             let args: ParenthesizedGenericArguments = input.parse()?;
//             let parenthesized = PathArguments::Parenthesized(args);
//             path.segments.last_mut().unwrap().arguments = parenthesized;
//         }

//         // Originally, the code was uncommented
//         //
//         //{
//         //    if let Some((tilde_token, const_token)) = tilde_const {
//         //        path.segments.insert(
//         //            0,
//         //            PathSegment {
//         //                ident: Ident::new("const", const_token.span),
//         //                arguments: PathArguments::None,
//         //            },
//         //        );
//         //        let (_const, punct) = path.segments.pairs_mut().next().unwrap().into_tuple();
//         //        *punct.unwrap() = Token![::](tilde_token.span);
//         //    }
//         //}

//         Ok(TraitBound {
//             paren_token: None,
//             modifier,
//             lifetimes,
//             path,
//         })
//     }
// }

// // generics.rs (syn 1.0.86)
// impl Parse for TypeParamBound {
//     fn parse(input: ParseStream) -> Result<Self> {
//         if input.peek(Lifetime) {
//             return input.parse().map(TypeParamBound::Lifetime);
//         }

//         if input.peek(Paren) {
//             let content;
//             let paren_token = parenthesized!(content in input);
//             let mut bound: TraitBound = content.parse::<TraitBound>()?;
//             bound.paren_token = Some(paren_token);
//             return Ok(TypeParamBound::Trait(bound));
//         }

//         input.parse::<TraitBound>().map(TypeParamBound::Trait)
//     }
// }

// // generics.rs (syn 1.0.86)
// // Originally, the code was generated with a macro
// impl ToTokens for TypeParamBound {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         match self {
//             TypeParamBound::Trait(t) => t.to_tokens(tokens),
//             TypeParamBound::Lifetime(l) => l.to_tokens(tokens),
//         }
//     }
// }

// // verbatim.rs (syn 1.0.86)
// mod verbatim {
//     use super::*;
//     pub fn between<'a>(begin: ParseBuffer<'a>, end: ParseStream<'a>) -> TokenStream2 {
//         let end = end.cursor();
//         let mut cursor = begin.cursor();
//         let mut tokens = TokenStream2::new();
//         while cursor != end {
//             let (tt, next) = cursor.token_tree().unwrap();
//             tokens.extend(core::iter::once(tt));
//             cursor = next;
//         }
//         tokens
//     }
// }

// // generics.rs (syn 1.0.86)
// impl Parse for TypeParam {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let attrs = input.call(Attribute::parse_outer)?;
//         let ident: Ident = input.parse()?;
//         let colon_token: Option<Token![:]> = input.parse()?;

//         let begin_bound = input.fork();
//         let mut is_maybe_const = false;
//         let mut bounds = Punctuated::new();
//         if colon_token.is_some() {
//             loop {
//                 if input.peek(Token![,]) || input.peek(Token![>]) || input.peek(Token![=]) {
//                     break;
//                 }
//                 let value: TypeParamBound = input.parse::<TypeParamBound>()?;

//                 match &value {
//                     TypeParamBound::Lifetime(_) => {},
//                     TypeParamBound::Trait(trait_) => {
//                         if let TraitBoundModifier::TildeConst(_) = trait_.modifier {
//                             is_maybe_const = true;
//                         }
//                     }
//                 }

//                 bounds.push_value(value);
//                 if !input.peek(Token![+]) {
//                     break;
//                 }
//                 let punct: Token![+] = input.parse()?;
//                 bounds.push_punct(punct);
//             }
//         }

//         let mut eq_token: Option<Token![=]> = input.parse()?;
//         let mut default = if eq_token.is_some() {
//             Some(input.parse::<Type>()?)
//         } else {
//             None
//         };

//         if is_maybe_const {
//             bounds.clear();
//             eq_token = None;
//             default = Some(Type::Verbatim(verbatim::between(begin_bound, input)));
//         }

//         Ok(TypeParam {
//             attrs,
//             ident,
//             colon_token,
//             bounds,
//             eq_token,
//             default,
//         })
//     }
// }

// // syn::attr (syn 1.0.86)
// impl<'a> FilterAttrs<'a> for &'a [Attribute] {
//     type Ret = core::iter::Filter<core::slice::Iter<'a, Attribute>, fn(&&Attribute) -> bool>;

//     fn outer(self) -> Self::Ret {
//         fn is_outer(attr: &&Attribute) -> bool {
//             match attr.style {
//                 AttrStyle::Outer => true,
//                 AttrStyle::Inner(_) => false,
//             }
//         }
//         self.iter().filter(is_outer)
//     }

//     fn inner(self) -> Self::Ret {
//         fn is_inner(attr: &&Attribute) -> bool {
//             match attr.style {
//                 AttrStyle::Inner(_) => true,
//                 AttrStyle::Outer => false,
//             }
//         }
//         self.iter().filter(is_inner)
//     }
// }

// // syn::attr (syn 1.0.86)
// trait FilterAttrs<'a> {
//     type Ret: Iterator<Item = &'a Attribute>;

//     fn outer(self) -> Self::Ret;
//     fn inner(self) -> Self::Ret;
// }

// // generics.rs (syn 1.0.86)
// // Originally, the code was generated with a macro
// impl ToTokens for TypeParam {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         tokens.append_all(self.attrs.outer());
//         self.ident.to_tokens(tokens);
//         if !self.bounds.is_empty() {
//             TokensOrDefault(&self.colon_token).to_tokens(tokens);
//             self.bounds.to_tokens(tokens);
//         }
//         if let Some(default) = &self.default {
//             TokensOrDefault(&self.eq_token).to_tokens(tokens);
//             default.to_tokens(tokens);
//         }
//     }
// }

// // generics.rs (syn 1.0.86)
// // Originally, the code was generated with a macro
// impl ToTokens for GenericParam {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         match self {
//             GenericParam::Type(_e) => _e.to_tokens(tokens),
//             GenericParam::Lifetime(_e) => _e.to_tokens(tokens),
//             GenericParam::Const(_e) => _e.to_tokens(tokens),
//         }
//     }
// }

// // generics.rs (syn 1.0.86)
// impl Parse for Generics {
//     fn parse(input: ParseStream) -> Result<Self> {
//         if !input.peek(Token![<]) {
//             return Ok(Generics::default());
//         }

//         let lt_token: Token![<] = input.parse()?;

//         let mut params = Punctuated::new();
//         loop {
//             if input.peek(Token![>]) {
//                 break;
//             }

//             let attrs = input.call(Attribute::parse_outer)?;
//             let lookahead = input.lookahead1();
//             if lookahead.peek(Lifetime) {
//                 params.push_value(GenericParam::Lifetime(LifetimeDef {
//                     attrs,
//                     ..input.parse()?
//                 }));
//             } else if lookahead.peek(Ident) {
//                 params.push_value(GenericParam::Type(TypeParam {
//                     attrs,
//                     ..input.parse::<TypeParam>()?
//                 }));
//             } else if lookahead.peek(Token![const]) {
//                 params.push_value(GenericParam::Const(ConstParam {
//                     attrs,
//                     ..input.parse::<ConstParam>()?
//                 }));
//             } else if input.peek(Token![_]) {
//                 params.push_value(GenericParam::Type(TypeParam {
//                     attrs,
//                     ident: input.call(Ident::parse_any)?,
//                     colon_token: None,
//                     bounds: Punctuated::new(),
//                     eq_token: None,
//                     default: None,
//                 }));
//             } else {
//                 return Err(lookahead.error());
//             }

//             if input.peek(Token![>]) {
//                 break;
//             }
//             let punct = input.parse()?;
//             params.push_punct(punct);
//         }

//         let gt_token: Token![>] = input.parse()?;

//         Ok(Generics {
//             lt_token: Some(lt_token),
//             params,
//             gt_token: Some(gt_token),
//             where_clause: None,
//         })
//     }
// }

// struct TokensOrDefault<'a, T: 'a>(pub &'a Option<T>);

// impl<'a, T> ToTokens for TokensOrDefault<'a, T>
// where
//     T: ToTokens + Default,
// {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         match self.0 {
//             Some(t) => t.to_tokens(tokens),
//             None => T::default().to_tokens(tokens),
//         }
//     }
// }

// // generics.rs (syn 1.0.86)
// impl ToTokens for Generics {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         if self.params.is_empty() {
//             return;
//         }

//         TokensOrDefault(&self.lt_token).to_tokens(tokens);

//         // Print lifetimes before types and consts, regardless of their
//         // order in self.params.
//         //
//         // TODO: ordering rules for const parameters vs type parameters have
//         // not been settled yet. https://github.com/rust-lang/rust/issues/44580
//         let mut trailing_or_empty = true;
//         for param in self.params.pairs() {
//             if let GenericParam::Lifetime(_) = **param.value() {
//                 <Pair<&GenericParam, &Comma> as ToTokens>::to_tokens(&param, tokens);
//                 trailing_or_empty = param.punct().is_some();
//             }
//         }
//         for param in self.params.pairs() {
//             match **param.value() {
//                 GenericParam::Type(_) | GenericParam::Const(_) => {
//                     if !trailing_or_empty {
//                         <Token![,]>::default().to_tokens(tokens);
//                         trailing_or_empty = true;
//                     }
//                     param.to_tokens(tokens);
//                 }
//                 GenericParam::Lifetime(_) => {}
//             }
//         }

//         TokensOrDefault(&self.gt_token).to_tokens(tokens);
//     }
// }

// syn::attr::parsing::parse_inner (syn 1.0.86)
fn single_parse_inner(input: ParseStream) -> Result<Attribute> {
    let content;
    Ok(Attribute {
        pound_token: input.parse::<Pound>()?,
        style: AttrStyle::Inner(input.parse::<Bang>()?),
        bracket_token: bracketed!(content in input),
        path: content.call(Path::parse_mod_style)?,
        tokens: content.parse::<TokenStream2>()?,
    })
}

// syn::attr::parsing::parse_inner (syn 1.0.86)
fn parse_inner(input: ParseStream, attrs: &mut Vec<Attribute>) -> Result<()> {
    while input.peek(Token![#]) && input.peek2(Token![!]) {
        attrs.push(input.call(single_parse_inner)?);
    }
    Ok(())
}

impl Parse for ItemConstImpl {
    // Largely based on: https://docs.rs/syn/1.0.86/src/syn/item.rs.html#2402-2407
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let defaultness = input.parse::<Option<DefaultKW>>()?;
        let unsafety = input.parse::<Option<Token![unsafe]>>()?;
        let impl_token = input.parse::<Impl>()?;

        let has_generics = input.peek(Token![<])
            && (input.peek2(Token![>])
                || input.peek2(Token![#])
                || (input.peek2(Ident) || input.peek2(Lifetime))
                    && (input.peek3(Token![:])
                        || input.peek3(Token![,])
                        || input.peek3(Token![>])
                        || input.peek3(Token![=]))
                || input.peek2(Token![const]));
        let mut generics: Generics = if has_generics {
            input.parse::<Generics>()?
        } else {
            Generics::default()
        };

        let is_const_impl = input.peek(Token![const]);
        // The author is uncertain where the second kind of const impl comes from
        // || input.peek(Token![?]) && input.peek2(Token![const]);
        let constness = if is_const_impl {
            // input.parse::<Option<Token![?]>>()?;
            Some(input.parse::<Token![const]>()?)
        } else {
            None
        };
        let polarity = if input.peek(Token![!]) && !input.peek2(Brace) {
            Some(input.parse::<Token![!]>()?)
        } else {
            None
        };
        let first_ty_span = input.span();
        let mut first_ty: Type = input.parse::<Type>()?;
        let self_ty: Type;
        let trait_;

        let is_impl_for = input.peek(Token![for]);
        if is_impl_for {
            let for_token: Token![for] = input.parse::<Token![for]>()?;
            let mut first_ty_ref = &first_ty;
            while let Type::Group(ty) = first_ty_ref {
                first_ty_ref = &ty.elem;
            }
            if let Type::Path(TypePath { qself: None, .. }) = first_ty_ref {
                while let Type::Group(ty) = first_ty {
                    first_ty = *ty.elem;
                }
                if let Type::Path(TypePath { qself: None, path }) = first_ty {
                    trait_ = Some((polarity, path, for_token));
                } else {
                    unreachable!();
                }
            } else {
                return Err(Error::new(first_ty_span, "expected trait path"));
            }
            self_ty = input.parse::<Type>()?;
        } else {
            return Err(Error::new(Span2::call_site(), "expected trait impl block"));
        };
        generics.where_clause = input.parse::<Option<WhereClause>>()?;

        let content;
        let brace_token = braced!(content in input);
        parse_inner(&content, &mut attrs)?;

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse::<ImplItem>()?);
        }
        if is_impl_for && trait_.is_none() {
            return Err(Error::new(is_impl_for.span(), "expected trait name"));
        } else {
            Ok(ItemConstImpl {
                attrs,
                defaultness,
                unsafety,
                impl_token,
                generics,
                constness,
                trait_,
                self_ty: Box::new(self_ty),
                brace_token,
                items,
            })
        }
    }
}

impl From<ItemConstImpl> for TokenStream {
    #[allow(unused_variables, clippy::let_and_return)]
    fn from(item_impl: ItemConstImpl) -> TokenStream {
        let ItemConstImpl {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            constness,
            trait_,
            self_ty,
            brace_token,
            items,
        } = item_impl;
        let mut ts = TokenStream::new();
        for attr in attrs.into_iter() {
            ts.extend::<TokenStream>(attr.to_token_stream().into());
        }
        ts.extend::<TokenStream>(defaultness.to_token_stream().into());
        ts.extend::<TokenStream>(unsafety.to_token_stream().into());
        ts.extend::<TokenStream>(impl_token.to_token_stream().into());
        ts.extend::<TokenStream>(generics.to_token_stream().into());
        ts.extend::<TokenStream>(constness.to_token_stream().into());
        match trait_ {
            None => {}
            Some((bang, path, for_)) => {
                ts.extend::<TokenStream>(bang.to_token_stream().into());
                ts.extend::<TokenStream>(path.to_token_stream().into());
                ts.extend::<TokenStream>(for_.to_token_stream().into());
            }
        };
        ts.extend::<TokenStream>(self_ty.to_token_stream().into());
        let mut nested_ts = TokenStream2::new();
        for item in items.into_iter() {
            nested_ts.extend(item.to_token_stream());
        }
        ts.extend::<TokenStream>(
            Group2::new(Delimiter2::Brace, nested_ts)
                .to_token_stream()
                .into(),
        );
        // let comment = format!("const S: &str = \"{}\";", ts);
        // let ts = <TokenStream as std::str::FromStr>::from_str(&comment).unwrap();
        ts
    }
}

#[proc_macro_attribute]
pub fn unconst_trait_impl(_attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let item_const_impl = parse_macro_input!(item as ItemConstImpl);
    item_const_impl.into()
}
