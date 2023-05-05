# Function-like macro that "unconsts" trait implementations

`unconst_trait_impl::unconst_trait_impl` turns the Nightly syntax for constant trait implementations into analogous non-const syntax that is accepted on stable toolchain.

The list of features taken into account:
* [const_trait_impl](https://doc.rust-lang.org/nightly/unstable-book/language-features/const-trait-impl.html)
* [const_fn_trait_bound](https://doc.rust-lang.org/nightly/unstable-book/language-features/const-fn-trait-bound.html)

In a vaccum, `unconst_trait_impl` [procedural function-like macro][proc macro] is fairly useless because its call on constant trait implementation yields the same result as writing the non-const implementation in the first place.

However, with [`cfg_attr`] and `remove_macro_call` [attributes][attribute], `unconst_trait_impl` macro allows one to **conditionally** remove the macro call thus providing support for stable toolchain while also providing functionality relying on Nightly features.

## Example

```rust, ignore
#![cfg_attr(feature = "const_trait_impl", feature(const_trait_impl))]
#![cfg_attr(feature = "const_default_impls", feature(const_default_impls))]
#![cfg_attr(feature = "const_fn_trait_bound", feature(const_fn_trait_bound))]

#[cfg(not(all(
    feature = "const_trait_impl",
    feature = "const_default_impls",
    feature = "const_fn_trait_bound"
)))]
use unconst_trait_impl::unconst_trait_impl;
use core::{default::Default, marker::PhantomData};
#[cfg(all(
    feature = "const_trait_impl",
    feature = "const_default_impls",
    feature = "const_fn_trait_bound"
))]
use remove_macro_call::remove_macro_call;

// Since ZST is both Eq and and PartialEq, it has structural match
// https://github.com/rust-lang/rust/issues/63438
#[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd, Copy)]
pub struct ZST<T: ?Sized>(PhantomData<T>);

pub trait TraitName {}

#[cfg_attr(
    all(
        feature = "const_trait_impl",
        feature = "const_default_impls",
        feature = "const_fn_trait_bound"
    ),
    remove_macro_call
)]
unconst_trait_impl! {
    impl<T: ?Sized> const TraitName for ZST<T> {}
}

// With `cargo build --features const_trait_impl, const_default_impls, const_fn_trait_bound`
// or with `cargo build --all-features, the code below is expanded as is. Otherwise,
// it gets "unconsted" to be supported by stable toolchain.
#[cfg_attr(
    all(
        feature = "const_trait_impl",
        feature = "const_default_impls",
        feature = "const_fn_trait_bound"
    ),
    remove_macro_call
)]
unconst_trait_impl! {
    impl<T: ~const TraitName + ?Sized> const Default for ZST<T> {
        fn default() -> Self {
            ZST(Default::default())
        }
    }
}
```

**Note**: In the real code, the example above could be replaced with a simpler version relying on [`cfg_aliases`](https://crates.io/crates/cfg_aliases) crate.

You can learn more about `remove_macro_call` here:
* [GitHub](https://github.com/JohnScience/remove_macro_call)
* [crates.io](https://crates.io/crates/remove_macro_call)

## Why is it so ugly?

From the standpoint of stable Rust, nightly Rust syntax is **not** Rust. Therefore, using an attribute would not suffice. 

[According to the reference](https://doc.rust-lang.org/reference/attributes.html), attributes may be applied to many things in the language:

* All item declarations accept outer attributes while external blocks, functions, implementations, and modules accept inner attributes.
* Most statements accept outer attributes (see Expression Attributes for limitations on expression statements).
* Block expressions accept outer and inner attributes, but only when they are the outer expression of an expression statement or the final expression of another block expression.
* Enum variants and struct and union fields accept outer attributes.
* Match expression arms accept outer attributes.
* Generic lifetime or type parameter accept outer attributes.
* Expressions accept outer attributes in limited situations, see Expression Attributes for details.
* Function, closure and function pointer parameters accept outer attributes. This includes attributes on variadic parameters denoted with ... in function pointers and external blocks.

Notice that "arbitrary Rust-like syntax" is not one of them. So the nightly syntax *has* to be wrapped in a function-like macro, which does permit arbitrary Rust-like syntax. It is also necessary to evaluate the [configuration predicates](https://doc.rust-lang.org/reference/conditional-compilation.html#:~:text=compilation%20takes%20a-,configuration%20predicate,-that%20evaluates%20to) before expansion of the function-like macro, which is problematic if at all possible in the context of a function-like macro.

## Known limitations

Currently, type parameters (like `T` in `T: ~const TraitName + ?Sized`) get "unconsted" only when

1. they belong to the trait implementation, i.e. when they are in `<..>` or in the `where` clause of the trait implementation;
2. they belong to the signatures of methods and associated functions;

Other [Items](https://docs.rs/syn/latest/syn/enum.ImplItem.html) in the trait implementation currently don't get "unconsted".

# License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

[attribute]: https://doc.rust-lang.org/reference/attributes.html
[proc macro]: https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/#functionlikemacros
[`cfg_attr`]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
