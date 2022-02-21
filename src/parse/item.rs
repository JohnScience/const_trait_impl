use crate::{ImplItem, TypeParamBound};
use syn::{
    parse::{Parse, ParseBuffer, ParseStream, Result},
    punctuated::Punctuated,
    Generics, Ident, ImplItemType, Type, Visibility,
};

#[allow(dead_code)]
struct FlexibleItemType {
    vis: Visibility,
    defaultness: Option<syn::token::Default>,
    type_token: syn::token::Type,
    ident: Ident,
    generics: Generics,
    colon_token: Option<syn::token::Colon>,
    bounds: Punctuated<TypeParamBound, syn::token::Add>,
    ty: Option<(syn::token::Eq, Type)>,
    semi_token: syn::token::Semi,
}

impl Parse for FlexibleItemType {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis: Visibility = input.parse()?;
        let defaultness: Option<syn::token::Default> = input.parse()?;
        let type_token: syn::token::Type = input.parse()?;
        let ident: Ident = input.parse()?;
        let mut generics: Generics = input.parse()?;
        let colon_token: Option<syn::token::Colon> = input.parse()?;
        let mut bounds = Punctuated::new();
        if colon_token.is_some() {
            loop {
                if input.peek(syn::token::Where)
                    || input.peek(syn::token::Eq)
                    || input.peek(syn::token::Semi)
                {
                    break;
                }
                bounds.push_value(input.parse::<TypeParamBound>()?);
                if input.peek(syn::token::Where)
                    || input.peek(syn::token::Eq)
                    || input.peek(syn::token::Semi)
                {
                    break;
                }
                bounds.push_punct(input.parse::<syn::token::Add>()?);
            }
        }
        generics.where_clause = input.parse()?;
        let ty = if let Some(eq_token) = input.parse()? {
            Some((eq_token, input.parse::<Type>()?))
        } else {
            None
        };
        let semi_token: syn::token::Semi = input.parse()?;
        Ok(FlexibleItemType {
            vis,
            defaultness,
            type_token,
            ident,
            generics,
            colon_token,
            bounds,
            ty,
            semi_token,
        })
    }
}

pub(super) mod verbatim {
    use proc_macro2::TokenStream as TokenStream2;
    use std::iter;
    use syn::parse::{ParseBuffer, ParseStream};
    pub fn between<'a>(begin: ParseBuffer<'a>, end: ParseStream<'a>) -> TokenStream2 {
        let end = end.cursor();
        let mut cursor = begin.cursor();
        let mut tokens = TokenStream2::new();
        while cursor != end {
            let (tt, next) = cursor.token_tree().unwrap();
            tokens.extend(iter::once(tt));
            cursor = next;
        }
        tokens
    }
}

pub(super) fn parse_impl_item_type(begin: ParseBuffer, input: ParseStream) -> Result<ImplItem> {
    let FlexibleItemType {
        vis,
        defaultness,
        type_token,
        ident,
        generics,
        colon_token,
        bounds: _,
        ty,
        semi_token,
    } = input.parse()?;
    if colon_token.is_some() || ty.is_none() {
        Ok(ImplItem::Verbatim(verbatim::between(begin, input)))
    } else {
        let (eq_token, ty) = ty.unwrap();
        Ok(ImplItem::Type(ImplItemType {
            attrs: Vec::new(),
            vis,
            defaultness,
            type_token,
            ident,
            generics,
            eq_token,
            ty,
            semi_token,
        }))
    }
}

pub(super) fn peek_signature(input: ParseStream) -> bool {
    let fork = input.fork();
    fork.parse::<Option<syn::token::Const>>().is_ok()
        && fork.parse::<Option<syn::token::Async>>().is_ok()
        && fork.parse::<Option<syn::token::Unsafe>>().is_ok()
        && fork.parse::<Option<syn::Abi>>().is_ok()
        && fork.peek(syn::token::Fn)
}
