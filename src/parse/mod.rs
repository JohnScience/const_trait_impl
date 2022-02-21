mod local;

use proc_macro2::{TokenStream as TokenStream2, Span as Span2};
use crate::{
    GenericParam, Generics, TildeConst, TraitBound, TraitBoundModifier, TypeParam, TypeParamBound,
    WhereClause, WherePredicate, PredicateType, PredicateLifetime, ItemConstImpl
};
use syn::{
    AttrStyle,
    braced,
    bracketed,
    ext::IdentExt,
    parenthesized,
    spanned::Spanned,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Paren, Default as DefaultKW, Impl, Pound, Bang, Brace},
    Attribute, BoundLifetimes, ConstParam, Ident, Lifetime, LifetimeDef,
    ParenthesizedGenericArguments, Path, PathArguments, Result, Token, Type, Error, TypePath, ImplItem
};
use local::LocalParse;

impl Parse for TildeConst {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            tilde: input.parse::<Token![~]>()?,
            const_: input.parse::<Token![const]>()?,
        })
    }
}

// generics.rs (syn 1.0.86)
impl Parse for TraitBoundModifier {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![?]) {
            input.parse().map(TraitBoundModifier::Maybe)
        } else if input.peek(Token![~]) && input.peek2(Token![const]) {
            input.parse().map(TraitBoundModifier::TildeConst)
        } else {
            Ok(TraitBoundModifier::None)
        }
    }
}

// generics.rs (syn 1.0.86)
impl Parse for TraitBound {
    fn parse(input: ParseStream) -> Result<Self> {
        let modifier: TraitBoundModifier = input.parse()?;
        let lifetimes: Option<BoundLifetimes> = input.parse()?;

        let mut path: Path = input.parse()?;
        if path.segments.last().unwrap().arguments.is_empty()
            && (input.peek(Paren) || input.peek(Token![::]) && input.peek3(Paren))
        {
            input.parse::<Option<Token![::]>>()?;
            let args: ParenthesizedGenericArguments = input.parse()?;
            let parenthesized = PathArguments::Parenthesized(args);
            path.segments.last_mut().unwrap().arguments = parenthesized;
        }

        // {
        //     if let TraitBoundModifier::TildeConst(TildeConst {
        //         tilde,
        //         const_,
        //     }) = modifier
        //     {
        //         path.segments.insert(
        //             0,
        //             PathSegment {
        //                 ident: Ident::new("const", const_.span),
        //                 arguments: PathArguments::None,
        //             },
        //         );
        //         let (_const, punct) = path.segments.pairs_mut().next().unwrap().into_tuple();
        //         *punct.unwrap() = Token![::](tilde.span);
        //     }
        // }

        Ok(TraitBound {
            paren_token: None,
            modifier,
            lifetimes,
            path,
        })
    }
}

// generics.rs (syn 1.0.86)
impl Parse for TypeParamBound {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Lifetime) {
            return input.parse().map(TypeParamBound::Lifetime);
        }

        if input.peek(Paren) {
            let content;
            let paren_token = parenthesized!(content in input);
            let mut bound: TraitBound = content.parse::<TraitBound>()?;
            bound.paren_token = Some(paren_token);
            return Ok(TypeParamBound::Trait(bound));
        }

        input.parse::<TraitBound>().map(TypeParamBound::Trait)
    }
}

// generics.rs (syn 1.0.86)
impl Parse for TypeParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;
        let colon_token: Option<Token![:]> = input.parse()?;

        // let begin_bound = input.fork();
        // let mut is_maybe_const = false;
        let mut bounds = Punctuated::new();
        if colon_token.is_some() {
            loop {
                if input.peek(Token![,]) || input.peek(Token![>]) || input.peek(Token![=]) {
                    break;
                }
                let value: TypeParamBound = input.parse::<TypeParamBound>()?;

                match &value {
                    TypeParamBound::Lifetime(_) => {}
                    TypeParamBound::Trait(_trait_) => {
                        //if let TraitBoundModifier::TildeConst(_) = trait_.modifier {
                        //    is_maybe_const = true;
                        //}
                    }
                }

                bounds.push_value(value);
                if !input.peek(Token![+]) {
                    break;
                }
                let punct: Token![+] = input.parse()?;
                bounds.push_punct(punct);
            }
        }

        let eq_token: Option<Token![=]> = input.parse()?;
        let default = if eq_token.is_some() {
            Some(input.parse::<Type>()?)
        } else {
            None
        };

        // if is_maybe_const {
        //     bounds.clear();
        //     eq_token = None;
        //     default = Some(Type::Verbatim(verbatim::between(begin_bound, input)));
        // }

        Ok(TypeParam {
            attrs,
            ident,
            colon_token,
            bounds,
            eq_token,
            default,
        })
    }
}

// generics.rs (syn 1.0.86)
impl Parse for Generics {
    fn parse(input: ParseStream) -> Result<Self> {
        if !input.peek(Token![<]) {
            return Ok(Generics::default());
        }

        let lt_token: Token![<] = input.parse()?;

        let mut params = Punctuated::new();
        loop {
            if input.peek(Token![>]) {
                break;
            }

            let attrs = input.call(Attribute::parse_outer)?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Lifetime) {
                params.push_value(GenericParam::Lifetime(LifetimeDef {
                    attrs,
                    ..input.parse()?
                }));
            } else if lookahead.peek(Ident) {
                params.push_value(GenericParam::Type(TypeParam {
                    attrs,
                    ..input.parse::<TypeParam>()?
                }));
            } else if lookahead.peek(Token![const]) {
                params.push_value(GenericParam::Const(ConstParam {
                    attrs,
                    ..input.parse::<ConstParam>()?
                }));
            } else if input.peek(Token![_]) {
                params.push_value(GenericParam::Type(TypeParam {
                    attrs,
                    ident: input.call(Ident::parse_any)?,
                    colon_token: None,
                    bounds: Punctuated::new(),
                    eq_token: None,
                    default: None,
                }));
            } else {
                return Err(lookahead.error());
            }

            if input.peek(Token![>]) {
                break;
            }
            let punct = input.parse()?;
            params.push_punct(punct);
        }

        let gt_token: Token![>] = input.parse()?;

        Ok(Generics {
            lt_token: Some(lt_token),
            params,
            gt_token: Some(gt_token),
            where_clause: None,
        })
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl Parse for WhereClause {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(WhereClause {
            where_token: input.parse()?,
            predicates: {
                let mut predicates = Punctuated::new();
                loop {
                    if input.is_empty()
                        || input.peek(syn::token::Brace)
                        || input.peek(syn::token::Comma)
                        || input.peek(syn::token::Semi)
                        || input.peek(syn::token::Colon) && !input.peek(syn::token::Colon2)
                        || input.peek(syn::token::Eq)
                    {
                        break;
                    }
                    let value = input.parse::<WherePredicate>()?;
                    predicates.push_value(value);
                    if !input.peek(syn::token::Comma) {
                        break;
                    }
                    let punct = input.parse()?;
                    predicates.push_punct(punct);
                }
                predicates
            },
        })
    }
}

// generics.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl Parse for WherePredicate {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Lifetime) && input.peek2(syn::token::Colon) {
            Ok(WherePredicate::Lifetime(PredicateLifetime {
                lifetime: input.parse()?,
                colon_token: input.parse()?,
                bounds: {
                    let mut bounds = Punctuated::new();
                    loop {
                        if input.is_empty()
                            || input.peek(syn::token::Brace)
                            || input.peek(syn::token::Comma)
                            || input.peek(syn::token::Semi)
                            || input.peek(syn::token::Colon)
                            || input.peek(syn::token::Eq)
                        {
                            break;
                        }
                        let value = input.parse()?;
                        bounds.push_value(value);
                        if !input.peek(syn::token::Add) {
                            break;
                        }
                        let punct = input.parse()?;
                        bounds.push_punct(punct);
                    }
                    bounds
                },
            }))
        } else {
            Ok(WherePredicate::Type(PredicateType {
                lifetimes: input.parse()?,
                bounded_ty: input.parse()?,
                colon_token: input.parse()?,
                bounds: {
                    let mut bounds = Punctuated::new();
                    loop {
                        if input.is_empty()
                            || input.peek(syn::token::Brace)
                            || input.peek(syn::token::Comma)
                            || input.peek(syn::token::Semi)
                            || input.peek(syn::token::Colon) && !input.peek(syn::token::Colon2)
                            || input.peek(syn::token::Eq)
                        {
                            break;
                        }
                        let value = input.parse()?;
                        bounds.push_value(value);
                        if !input.peek(syn::token::Add) {
                            break;
                        }
                        let punct = input.parse()?;
                        bounds.push_punct(punct);
                    }
                    bounds
                },
            }))
        }
    }
}

// syn::attr::parsing::parse_inner (syn 1.0.86)
#[allow(clippy::eval_order_dependence)]
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
        generics.where_clause = Option::<WhereClause>::local_parse(input)?;

        let content;
        let brace_token = braced!(content in input);
        parse_inner(&content, &mut attrs)?;

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse::<ImplItem>()?);
        }
        if is_impl_for && trait_.is_none() {
            Err(Error::new(is_impl_for.span(), "expected trait name"))
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
