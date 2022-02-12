use proc_macro::TokenStream;
use proc_macro2::{Delimiter as Delimiter2, Group as Group2, TokenStream as TokenStream2, Span as Span2};
use quote::ToTokens;
use syn::{
    parse_macro_input,
    token::{Bang, Brace, Const, Default, For, Impl, Unsafe, Lt, Gt, Comma},
    Result, Attribute, ImplItem, Path, Type, parse::{Parse, ParseStream}, punctuated::Punctuated, GenericParam, WhereClause, Token, Ident, Lifetime, Generics, TypePath, Error, braced, AttrStyle, bracketed, spanned::Spanned,
};

// syn::Generics is outdated because syn::GenericParam is. 

// struct Generics {
//     lt_token: Option<Lt>,
//     params: Punctuated<GenericParam, Comma>,
//     gt_token: Option<Gt>,
//     where_clause: Option<WhereClause>,
// }

struct ItemConstImpl {
    attrs: Vec<Attribute>,
    // https://github.com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md
    defaultness: Option<Default>,
    unsafety: Option<Unsafe>,
    impl_token: Impl,
    generics: Generics,
    constness: Option<Const>,
    trait_: Option<(Option<Bang>, Path, For)>,
    self_ty: Box<Type>,
    brace_token: Brace,
    items: Vec<ImplItem>,
}

// syn::attr::parsing::parse_inner (syn 1.0.86)
fn single_parse_inner(input: ParseStream) -> Result<Attribute> {
    let content;
    Ok(Attribute {
        pound_token: input.parse()?,
        style: AttrStyle::Inner(input.parse()?),
        bracket_token: bracketed!(content in input),
        path: content.call(Path::parse_mod_style)?,
        tokens: content.parse()?,
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
        let defaultness = input.parse::<Option<Default>>()?;
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

        let is_const_impl =
            input.peek(Token![const]);
            // The author is uncertain where the second kind of const impl comes from
            // || input.peek(Token![?]) && input.peek2(Token![const]);
        let constness = if is_const_impl {
            // input.parse::<Option<Token![?]>>()?;
            Some(input.parse::<Token![const]>()?)
        } else { None };
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
                if let Type::Path(TypePath { qself: None, path}) = first_ty {
                    trait_ = Some((polarity, path, for_token));
                } else {
                    unreachable!();
                }
            }
            else {
                return Err(Error::new(first_ty_span, "expected trait path"));
            }
            self_ty = input.parse()?;
        } else {
            return Err(Error::new(Span2::call_site(), "expected trait impl block"));
        };
        generics.where_clause = input.parse()?;

        let content;
        let brace_token = braced!(content in input);
        parse_inner(&content, &mut attrs)?;

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
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
        // TokenStream reorders the supplied tokens.
        // No idea what to do
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
pub fn const_trait_impl(_attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let item_const_impl = parse_macro_input!(item as ItemConstImpl);
    item_const_impl.into()
}
