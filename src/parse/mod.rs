mod item;
mod local;

use crate::{
    GenericParam, Generics, ImplItem, ImplItemMethod, ItemConstImpl, PredicateLifetime,
    PredicateType, Signature, TildeConst, TraitBound, TraitBoundModifier, TypeParam,
    TypeParamBound, WhereClause, WherePredicate,
};
use item::{parse_impl_item_type, peek_signature, verbatim};
use local::{LocalIsInherited, LocalParse};
use proc_macro2::{
    Punct, Spacing, Span as Span2, TokenStream as TokenStream2, TokenTree as TokenTree2,
};
use syn::{
    braced, bracketed,
    ext::IdentExt,
    parenthesized,
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Bang, Brace, Default as DefaultKW, Impl, Paren, Pound},
    Abi, AttrStyle, Attribute, Block, BoundLifetimes, ConstParam, Error, FnArg, Ident,
    ImplItemConst, Item, Lifetime, LifetimeDef, ParenthesizedGenericArguments, Pat, PatType, Path,
    PathArguments, Result, ReturnType, Stmt, Token, Type, TypePath, Variadic, Visibility,
};

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
            items.push(ImplItem::parse(&content)?);
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

// item.rs (syn 1.0.86)
// Originally, the code was generated with a macro
impl Parse for ImplItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let begin = input.fork();
        let mut attrs = input.call(Attribute::parse_outer)?;
        let ahead = input.fork();
        let vis: Visibility = ahead.parse()?;
        let mut lookahead = ahead.lookahead1();
        let defaultness = if lookahead.peek(syn::token::Default) && !ahead.peek2(syn::token::Bang) {
            let defaultness: syn::token::Default = ahead.parse()?;
            lookahead = ahead.lookahead1();
            Some(defaultness)
        } else {
            None
        };
        let mut item = if lookahead.peek(syn::token::Fn) || peek_signature(&ahead) {
            input.parse().map(ImplItem::Method)
        } else if lookahead.peek(syn::token::Const) {
            let const_token: syn::token::Const = ahead.parse()?;
            let lookahead = ahead.lookahead1();
            if lookahead.peek(Ident) || lookahead.peek(syn::token::Underscore) {
                input.advance_to(&ahead);
                let ident: Ident = input.call(Ident::parse_any)?;
                let colon_token: syn::token::Colon = input.parse()?;
                let ty: Type = input.parse()?;
                if let Some(eq_token) = input.parse()? {
                    return Ok(ImplItem::Const(ImplItemConst {
                        attrs,
                        vis,
                        defaultness,
                        const_token,
                        ident,
                        colon_token,
                        ty,
                        eq_token,
                        expr: input.parse()?,
                        semi_token: input.parse()?,
                    }));
                } else {
                    input.parse::<syn::token::Semi>()?;
                    return Ok(ImplItem::Verbatim(verbatim::between(begin, input)));
                }
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(syn::token::Type) {
            parse_impl_item_type(begin, input)
        } else if vis.local_is_inherited()
            && defaultness.is_none()
            && (lookahead.peek(Ident)
                || lookahead.peek(syn::token::SelfValue)
                || lookahead.peek(syn::token::Super)
                || lookahead.peek(syn::token::Crate)
                || lookahead.peek(syn::token::Colon2))
        {
            input.parse().map(ImplItem::Macro)
        } else {
            Err(lookahead.error())
        }?;
        {
            let item_attrs = match &mut item {
                ImplItem::Const(item) => &mut item.attrs,
                ImplItem::Method(item) => &mut item.attrs,
                ImplItem::Type(item) => &mut item.attrs,
                ImplItem::Macro(item) => &mut item.attrs,
                ImplItem::Verbatim(_) => return Ok(item),
                // _ => panic!("internal error: entered unreachable code"),
            };
            attrs.append(item_attrs);
            *item_attrs = attrs;
        }
        Ok(item)
    }
}

fn variadic_to_tokens(dots: &syn::token::Dot3) -> TokenStream2 {
    TokenStream2::from_iter(<[_]>::into_vec(Box::new([
        TokenTree2::Punct({
            let mut dot = Punct::new('.', Spacing::Joint);
            dot.set_span(dots.spans[0]);
            dot
        }),
        TokenTree2::Punct({
            let mut dot = Punct::new('.', Spacing::Joint);
            dot.set_span(dots.spans[1]);
            dot
        }),
        TokenTree2::Punct({
            let mut dot = Punct::new('.', Spacing::Alone);
            dot.set_span(dots.spans[2]);
            dot
        }),
    ])))
}

fn parse_fn_args(input: ParseStream) -> Result<Punctuated<FnArg, syn::token::Comma>> {
    let mut args = Punctuated::new();
    let mut has_receiver = false;
    while !input.is_empty() {
        let attrs = input.call(Attribute::parse_outer)?;
        let arg = if let Some(dots) = input.parse::<Option<syn::token::Dot3>>()? {
            FnArg::Typed(PatType {
                attrs,
                pat: Box::new(Pat::Verbatim(variadic_to_tokens(&dots))),
                colon_token: syn::token::Colon(dots.spans[0]),
                ty: Box::new(Type::Verbatim(variadic_to_tokens(&dots))),
            })
        } else {
            let mut arg: FnArg = input.parse()?;
            match &mut arg {
                FnArg::Receiver(receiver) if has_receiver => {
                    return Err(Error::new(
                        receiver.self_token.span,
                        "unexpected second method receiver",
                    ));
                }
                FnArg::Receiver(receiver) if !args.is_empty() => {
                    return Err(Error::new(
                        receiver.self_token.span,
                        "unexpected method receiver",
                    ));
                }
                FnArg::Receiver(receiver) => {
                    has_receiver = true;
                    receiver.attrs = attrs;
                }
                FnArg::Typed(arg) => arg.attrs = attrs,
            }
            arg
        };
        args.push_value(arg);
        if input.is_empty() {
            break;
        }
        let comma: syn::token::Comma = input.parse()?;
        args.push_punct(comma);
    }
    Ok(args)
}

fn pop_variadic(args: &mut Punctuated<FnArg, syn::token::Comma>) -> Option<Variadic> {
    let trailing_punct = args.trailing_punct();
    let last = match args.last_mut()? {
        FnArg::Typed(last) => last,
        _ => return None,
    };
    let ty = match last.ty.as_ref() {
        Type::Verbatim(ty) => ty,
        _ => return None,
    };
    let mut variadic = Variadic {
        attrs: Vec::new(),
        dots: parse2(ty.clone()).ok()?,
    };
    if let Pat::Verbatim(pat) = last.pat.as_ref() {
        if pat.to_string() == "..." && !trailing_punct {
            variadic.attrs = core::mem::replace(&mut last.attrs, Vec::new());
            args.pop();
        }
    }
    Some(variadic)
}

impl Parse for Signature {
    fn parse(input: ParseStream) -> Result<Self> {
        let constness: Option<syn::token::Const> = input.parse()?;
        let asyncness: Option<syn::token::Async> = input.parse()?;
        let unsafety: Option<syn::token::Unsafe> = input.parse()?;
        let abi: Option<Abi> = input.parse()?;
        let fn_token: syn::token::Fn = input.parse()?;
        let ident: Ident = input.parse()?;
        let mut generics: Generics = input.parse()?;
        let content;
        let paren_token = match syn::group::parse_parens(&input) {
            Result::Ok(parens) => {
                content = parens.content;
                parens.token
            }
            Result::Err(error) => {
                return Result::Err(error);
            }
        };
        let mut inputs = parse_fn_args(&content)?;
        let variadic = pop_variadic(&mut inputs);
        let output: ReturnType = input.parse()?;
        generics.where_clause = Option::<WhereClause>::local_parse(input)?;
        Ok(Signature {
            constness,
            asyncness,
            unsafety,
            abi,
            fn_token,
            ident,
            generics,
            paren_token,
            inputs,
            variadic,
            output,
        })
    }
}

impl Parse for ImplItemMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let defaultness: Option<syn::token::Default> = input.parse()?;
        let sig: Signature = input.parse()?;
        let block = if let Some(semi) = input.parse::<Option<syn::token::Semi>>()? {
            let mut punct = Punct::new(';', Spacing::Alone);
            punct.set_span(semi.span);
            let tokens =
                TokenStream2::from_iter(<[_]>::into_vec(Box::new([TokenTree2::Punct(punct)])));
            Block {
                brace_token: Brace { span: semi.span },
                stmts: <[_]>::into_vec(Box::new([Stmt::Item(Item::Verbatim(tokens))])),
            }
        } else {
            let content;
            let brace_token = match syn::group::parse_braces(&input) {
                Result::Ok(braces) => {
                    content = braces.content;
                    braces.token
                }
                Result::Err(error) => {
                    return Result::Err(error);
                }
            };
            attrs.extend(content.call(Attribute::parse_inner)?);
            Block {
                brace_token,
                stmts: content.call(Block::parse_within)?,
            }
        };
        Ok(ImplItemMethod {
            attrs,
            vis,
            defaultness,
            sig,
            block,
        })
    }
}
