<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. --> 

<!-- This `README.md` file is automatically generated from `docs.md`, which uses `rustdoc`'s syntax
   - to provide documentation for the `#[cfg_attrs { ... }]` macro too.
   -
   - See `build.rs` if you're interested to see the code, or edit `docs.md` to edit the
   - documentation. --> 

# `#[cfg_attrs { ... }]`
Provides an alternative syntax to [`#[cfg_attr(...)]`][cfg_attr] that is easier to use with doc
comments.

> <sup>Syntax</sup> \
> _CfgAttrsAttribute_ : \
> &nbsp;&nbsp;`cfg_attrs`
>
> _Attribute_ : \
> &nbsp;&nbsp;_ConfigureAttribute_ | [_OuterAttribute_]
>
> _ConfigureAttribute_ : \
> &nbsp;&nbsp; `#` `[` `configure` `(` _ConfigureMeta_ `)` `]`
>
> _ConfigureMeta_ : \
> &nbsp;&nbsp; [_ConfigurationPredicate_] `,` _Attributes_
>
> _Attributes_ : \
> &nbsp;&nbsp;_Attribute_<sup>\*</sup> ( `,` _Attribute_<sup>\*</sup> )<sup>\*</sup> `,`<sup>?</sup>

[_ConfigurationPredicate_]: https://doc.rust-lang.org/reference/conditional-compilation.html
[_OuterAttribute_]: https://doc.rust-lang.org/reference/attributes.html

## Usage
Placing `#[cfg_attrs]` on an item enables a `#[configure(<condition>, <attributes>)]` helper
attribute to be used on that item.

The syntax of that `#[configure(...)]` attribute is much like [`#[cfg_attr(...)]`][cfg_attr], except
the configured attributes use full attribute syntax. The advantage of this is that doc comments,
which expand to `#[doc = "..."]` attributes, can be used in the `#[configure(...)]` syntax.

## Examples
```rust
#[cfg_attrs]
/// This is an example struct.
#[configure(
    debug_assertions,
    ///
    /// Hello! These are docs that only appear when
    /// debug assertions are active.
)]
enum Example {
    #[configure(
        feature = "magic",
        /// Woah! Look at that! It enables
        /// `#[configure(...)]` for variants too!
    )]
    Point {
        #[configure(
            feature = "magic",
            /// And fields! This is amazing!
        )]
        x: i32,
        y: i32,
    },
}
```
This will expand to the following usage of [`#[cfg_attr(...)]`][cfg_attr]:
```rust
/// This is an example enum.
#[cfg_attr(
    debug_assertions,
    doc = "",
    doc = " Hello! These are docs that only appear when",
    doc = " debug assertions are active."
)]
enum Example {
    #[cfg_attr(
        feature = "magic",
        doc = " Woah! Look at that! It enables",
        doc = " `#[configure(...)]` for variants too!"
    )]
    Point {
        #[cfg_attr(
            feature = "magic",
            doc = " And fields! This is amazing!"
        )]
        x: i32,
        y: i32,
    },
}
```
Which, if debug assertions are active, would be expanded to:
```rust
/// This is an example enum.
///
/// Hello! These are docs that only appear when
/// debug assertions are active.
enum Example {
    Point {
        x: i32,
        y: i32,
    },
}
```
Or, if the `magic` feature is enabled:
```rust
/// This is an example enum.
enum Example {
    /// Woah! Look at that! It enables
    /// `#[configure(...)]` for variants too!
    Point {
        /// And fields! This is amazing!
        x: i32,
        y: i32,
    },
}
```

`#[cfg_attrs(...)]` may also be used with attributes other than doc comments, though there is
no real benefit to doing this:
```rust
#[cfg_attrs]
#[configure(
    feature = "magic",
    #[sparkles]
    #[crackles]
)]
fn bewitched() {}
```
With that example expanding to:
```rust
#[cfg_attr(feature = "magic", sparkles, crackles)]
fn bewitched() {}
```
And expanding, if the `magic` feature is enabled, to:
```rust ignore
#[sparkles]
#[crackles]
fn bewitched() {}
```

[cfg_attr]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
