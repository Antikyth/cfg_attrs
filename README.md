<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

# `#[cfg_attrs { ... }]`

Provides an alternative syntax to [`#[cfg_attr(...)]`][cfg_attr] that is easier to use with doc
comments.

> <sup>Syntax</sup> \
> _CfgAttrsAttribute_ : \
> &nbsp;&nbsp;`cfg_attrs` _CfgAttrs_
>
> _CfgAttrs_ : \
> &nbsp;&nbsp;( `{` _Attributes_ `}` ) | _ConfiguredAttrs_
>
> _ConfiguredAttrs_ : \
> &nbsp;&nbsp;`(` [_ConfigurationPredicate_] `,` _Attributes_ `)`
>
> _Attributes_ : \
> &nbsp;&nbsp;[_OuterAttribute_]<sup>\*</sup> ( `,` [_OuterAttribute_]<sup>\*</sup> )<sup>\*</sup> `,`<sup>?</sup>

[_ConfigurationPredicate_]: https://doc.rust-lang.org/reference/conditional-compilation.html
[_OuterAttribute_]: https://doc.rust-lang.org/reference/attributes.html

## Usage
`#[cfg_attrs { ... }]` should surround all other attributes on the item. A
`#[configure(<condition>, <attributes>)]` helper attribute is provided within.

The syntax of that `#[configure(...)]` attribute is much like [`#[cfg_attr(...)]`][cfg_attr], except
the configured attributes use full attribute syntax. The advantage of this is that doc comments,
which expand to `#[doc = "..."]` attributes, can be used in the `#[configure(...)]` syntax.

<div class="warning">

All of an item's doc comments should be placed within the `#[cfg_attrs { ... }]` attribute, even
if they are not being configured. Additionally, the `#[cfg_attrs { ... }]` attribute should only
appear once per item; `#[configure(...)]` can be used multiple times within it if you want
multiple usages.

</div>

These restrictions are as a result of how proc-macro attributes work: they are expanded
separately to other attributes, so their position among other attributes is lost. While you
might put a `#[cfg_attrs { ... }]` attribute that configures doc comments between two
non-configured doc comments, that isn't where it will be expanded to, so the documentation will
be out of order.

## Examples
```rust
#[cfg_attrs {
    /// This is an example struct.
    #[configure(
        debug_assertions,
        ///
        /// Hello! These are docs that only appear when
        /// debug assertions are active.
    )]
}]
struct Example;
```
This will expand to the following usage of [`#[cfg_attr(...)]`][cfg_attr]:
```rust
/// This is an example struct.
#[cfg_attr(
    debug_assertions,
    doc = "",
    doc = " Hello! These are docs that only appear when",
    doc = " debug assertions are active."
)]
struct Example;
```
Which, if debug assertions are active, would be expanded to:
```rust
/// This is an example struct.
///
/// Hello! These are docs that only appear when
/// debug assertions are active.
struct Example;
```

`#[cfg_attrs(...)]` may also be used with attributes other than doc comments, though there is
no real benefit to doing this:
```rust
#[cfg_attrs {
    #[configure(
        feature = "magic",
        #[sparkles]
        #[crackles]
    )]
}]
fn bewitched() {}
```
With that example being equivalent to:
```rust
#[cfg_attr(feature = "magic", sparkles, crackles)]
fn bewitched() {}
```

[cfg_attr]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
