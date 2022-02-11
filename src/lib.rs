use proc_macro::{TokenStream};
use proc_macro2::{Group, Delimiter, TokenStream as TokenStream2};
use syn::{parse_macro_input, ItemImpl, Attribute, token::{Default, Unsafe, Impl, Bang, For, Brace, Const}, Generics, Path, Type, ImplItem};
use quote::ToTokens;

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
    items: Vec<ImplItem>
}

impl From<ItemImpl> for ItemConstImpl {
    fn from(item_impl: ItemImpl) -> ItemConstImpl {
        ItemConstImpl {
            attrs: item_impl.attrs,
            defaultness: item_impl.defaultness,
            unsafety: item_impl.unsafety,
            impl_token: item_impl.impl_token,
            generics: item_impl.generics,
            constness: Some(Const::default()),
            trait_: item_impl.trait_,
            self_ty: item_impl.self_ty,
            brace_token: item_impl.brace_token,
            items: item_impl.items
        }
    }
}

// impl Parse for ItemConstImpl {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let attrs = input.call(Attribute::parse_outer)?;
//         let defaultness = input.parse::<Option<Default>>()?;
//         let unsafety = input.parse::<Unsafe>()?;
//         let impl_token = input.parse::<Impl>()?;
//     }
// }

impl From<ItemConstImpl> for TokenStream {
    #[allow(unused_variables)]
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
            items
        } = item_impl;
        // TokenStream reorders the supplied tokens.
        // No idea what to do
        let mut ts = TokenStream::new();
        for attr in attrs.into_iter() {
            ts.extend::<TokenStream>(attr.to_token_stream().into());
        };
        ts.extend::<TokenStream>(defaultness.to_token_stream().into());
        ts.extend::<TokenStream>(unsafety.to_token_stream().into());
        ts.extend::<TokenStream>(impl_token.to_token_stream().into());
        ts.extend::<TokenStream>(generics.to_token_stream().into());
        ts.extend::<TokenStream>(constness.to_token_stream().into());
        match trait_ {
            None => {},
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
        };
        ts.extend::<TokenStream>(Group::new(Delimiter::Brace, nested_ts).to_token_stream().into());
        let comment = format!("const S: &str = \"{}\";", ts);
        let ts = <TokenStream as std::str::FromStr>::from_str(&comment).unwrap();
        ts
    }
}

#[proc_macro_attribute]
pub fn const_trait_impl(_attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(item as ItemImpl);
    let item_const_impl: ItemConstImpl = item_impl.into();
    item_const_impl.into()
}
