#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse_macro_input,
    punctuated::{Pair, Punctuated},
    token::{
        Bang, Brace, Comma, Const, Default as DefaultKW, For, Gt, Impl, Lt, Paren, Unsafe,
    },
    Attribute, BoundLifetimes, ConstParam, Ident, ItemImpl, Lifetime, LifetimeDef, Path,
    PredicateEq, PredicateLifetime, Token, Type,
};
// syn::Generics is not suitable for support of const_trait_impl and const_fn_trait_bound
// due to the two transitive chains:
//
// use syn::Generics;
// use syn::GenericParam;
// use syn::TypeParam;
// use syn::TypeParamBound;
// use syn::TraitBound;
// use syn::TraitBoundModifier;
//
// use syn::Generics;
// use syn::WhereClause;
// use syn::WherePredicate;
// use syn::PredicateType;
// use syn::TypeParamBound;
//
use syn::ImplItem;
//
// TODO: track issue: <https://github.com/dtolnay/syn/issues/1130>

mod from;
mod parse;
mod to_tokens;

pub(crate) struct ItemConstImpl {
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

// enum ImplItem {
//     /// An associated constant within an impl block.
//     Const(ImplItemConst),
//
//     /// A method within an impl block.
//     Method(ImplItemMethod),
//
//     /// An associated type within an impl block.
//     Type(ImplItemType),
//
//     /// A macro invocation within an impl block.
//     Macro(ImplItemMacro),
//
//     /// Tokens within an impl block not interpreted by Syn.
//     Verbatim(TokenStream),
//
//     // // The following is the only supported idiom for exhaustive matching of
//     // // this enum.
//     // //
//     // //     match expr {
//     // //         ImplItem::Const(e) => {...}
//     // //         ImplItem::Method(e) => {...}
//     // //         ...
//     // //         ImplItem::Verbatim(e) => {...}
//     // //
//     // //         #[cfg(test)]
//     // //         ImplItem::__TestExhaustive(_) => unimplemented!(),
//     // //         #[cfg(not(test))]
//     // //         _ => { /* some sane fallback */ }
//     // //     }
//     // //
//     // // This way we fail your tests but don't break your library when adding
//     // // a variant. You will be notified by a test failure when a variant is
//     // // added, so that you can add code to handle it, but your library will
//     // // continue to compile and work for downstream users in the interim.
//     // //
//     // // Once `deny(reachable)` is available in rustc, ImplItem will be
//     // // reimplemented as a non_exhaustive enum.
//     // // https://github.com/rust-lang/rust/issues/44109#issuecomment-521781237
//     // #[doc(hidden)]
//     // __TestExhaustive(crate::private),
// }

// generics.rs (syn 1.0.86)
#[derive(Default)]
pub(crate) struct Generics {
    lt_token: Option<Lt>,
    params: Punctuated<GenericParam, Comma>,
    gt_token: Option<Gt>,
    where_clause: Option<WhereClause>,
}

// generics.rs (syn 1.0.86)
#[allow(clippy::large_enum_variant)]
pub(crate) enum GenericParam {
    /// A generic type parameter: `T: Into<String>`.
    Type(TypeParam),

    /// A lifetime definition: `'a: 'b + 'c + 'd`.
    Lifetime(LifetimeDef),

    /// A const generic parameter: `const LENGTH: usize`.
    Const(ConstParam),
}

// generics.rs (syn 1.0.86)
struct TypeParam {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub colon_token: Option<Token![:]>,
    pub bounds: Punctuated<TypeParamBound, Token![+]>,
    pub eq_token: Option<Token![=]>,
    pub default: Option<Type>,
}

// generics.rs (syn 1.0.86)
pub(crate) enum TypeParamBound {
    Trait(TraitBound),
    Lifetime(Lifetime),
}

// generics.rs (syn 1.0.86)
pub(crate) struct TraitBound {
    pub paren_token: Option<Paren>,
    pub modifier: TraitBoundModifier,
    /// The `for<'a>` in `for<'a> Foo<&'a T>`
    pub lifetimes: Option<BoundLifetimes>,
    /// The `Foo<&'a T>` in `for<'a> Foo<&'a T>`
    pub path: Path,
}

// generics.rs (syn 1.0.86)
pub(crate) enum TraitBoundModifier {
    None,
    Maybe(Token![?]),
    TildeConst(TildeConst),
}

pub(crate) struct TildeConst {
    tilde: Token![~],
    const_: Token![const],
}

pub(crate) struct PredicateType {
    /// Any lifetimes from a `for` binding
    pub lifetimes: Option<BoundLifetimes>,
    /// The type being bounded
    pub bounded_ty: Type,
    pub colon_token: Token![:],
    /// Trait and lifetime bounds (`Clone+Send+'static`)
    pub bounds: Punctuated<TypeParamBound, Token![+]>,
}

// generics.rs (syn 1.0.86)
pub(crate) enum WherePredicate {
    /// A type predicate in a `where` clause: `for<'c> Foo<'c>: Trait<'c>`.
    Type(PredicateType),

    /// A lifetime predicate in a `where` clause: `'a: 'b + 'c`.
    Lifetime(PredicateLifetime),

    /// An equality predicate in a `where` clause (unsupported).
    #[allow(dead_code)]
    Eq(PredicateEq),
}

// generics.rs (syn 1.0.86)
pub(crate) struct WhereClause {
    pub where_token: Token![where],
    pub predicates: Punctuated<WherePredicate, Token![,]>,
}

pub(crate) struct TokensOrDefault<'a, T: 'a>(pub &'a Option<T>);

// trait ToDbgString {
//     fn to_dbg_string(&self) -> String;
// }
//
// impl<T> ToDbgString for T
// where
//     T: ToTokens
// {
//     fn to_dbg_string(&self) -> String {
//         let ts = self.to_token_stream();
//         let t_name = core::any::type_name::<T>();
//         format!("{t_name}{{{ts}}}")
//     }
// }

/// Unconditionally turns const trait implementation into non-const
///
/// # Example
///
/// ```rust, ignore
/// #![cfg_attr(feature = "const_trait_impl", feature(const_trait_impl))]
/// #![cfg_attr(feature = "const_default_impls", feature(const_default_impls))]
/// #![cfg_attr(feature = "const_fn_trait_bound", feature(const_fn_trait_bound))]
///
/// #[cfg(not(all(
///     feature = "const_trait_impl",
///     feature = "const_default_impls",
///     feature = "const_fn_trait_bound"
/// )))]
/// use unconst_trait_impl::unconst_trait_impl;
/// use core::{default::Default, marker::PhantomData};
/// #[cfg(all(
///     feature = "const_trait_impl",
///     feature = "const_default_impls",
///     feature = "const_fn_trait_bound"
/// ))]
/// use remove_macro_call::remove_macro_call;
///
/// // Since ZST is both Eq and and PartialEq, it has structural match
/// // https://github.com/rust-lang/rust/issues/63438
/// #[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd, Copy)]
/// pub struct ZST<T: ?Sized>(PhantomData<T>);
///
/// pub trait TraitName {}
///
/// #[cfg_attr(
///     all(
///         feature = "const_trait_impl",
///         feature = "const_default_impls",
///         feature = "const_fn_trait_bound"
///     ),
///     remove_macro_call
/// )]
/// unconst_trait_impl! {
///     impl<T: ?Sized> const TraitName for ZST<T> {}
/// }
///
/// // With `cargo build --features const_trait_impl, const_default_impls, const_fn_trait_bound`
/// // or with `cargo build --all-features, the code below is expanded as is. Otherwise,
/// // it gets "unconsted" to be supported by stable toolchain.
/// #[cfg_attr(
///     all(
///         feature = "const_trait_impl",
///         feature = "const_default_impls",
///         feature = "const_fn_trait_bound"
///     ),
///     remove_macro_call
/// )]
/// unconst_trait_impl! {
///     impl<T: ~const TraitName + ?Sized> const Default for ZST<T> {
///         fn default() -> Self {
///             ZST(Default::default())
///         }
///     }
/// }
/// ```
///
/// **Note**: In the real code, the example above could be replaced with a simpler version relying on [`cfg_aliases`](https://crates.io/crates/cfg_aliases) crate.
///
/// You can learn more about `remove_macro_call` here:
/// * [GitHub](https://github.com/JohnScience/remove_macro_call)
/// * [crates.io](https://crates.io/crates/remove_macro_call)

#[proc_macro]
pub fn unconst_trait_impl(item: TokenStream) -> TokenStream {
    let item_const_impl = parse_macro_input!(item as ItemConstImpl);
    let item_impl: ItemImpl = item_const_impl.into();

    item_impl.to_token_stream().into()

    // let ItemImpl {
    //     attrs,
    //     defaultness,
    //     unsafety,
    //     impl_token,
    //     generics,
    //     trait_,
    //     self_ty,
    //     brace_token,
    //     items
    // } = item_impl;
    // let comment = format!("const S: &str = \"{}\";", generics.to_dbg_string());
    // let ts = <TokenStream as std::str::FromStr>::from_str(&comment).unwrap();
    // ts
}
