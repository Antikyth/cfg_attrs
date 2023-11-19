<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

# WARNING
This project is archived because it turns out that using this attribute will place doc comments
using it before all other doc comments 100% of the time. This is not something that can be changed,
because it's how attribute proc macros work.

# `#[cfg_attrs(...)]`

Provides an alternative syntax to [`#[cfg_attr(...)]`][cfg_attr] that is easier to use with doc
comments.

> <sup>Syntax</sup> \
> _CfgAttrsAttribute_ : \
> `cfg_attrs` `(` [_ConfigurationPredicate_] `,` _CfgAttrs_ `)`
>
> _CfgAttrs_ : \
> `{` _Attributes_ `}` | _Attributes_
>
> _Attributes_ : \
> [_OuterAttribute_]<sup>\*</sup> ( `,` [_OuterAttribute_]<sup>\*</sup> )<sup>\*</sup> `,`<sup>?</sup>

[_ConfigurationPredicate_]: https://doc.rust-lang.org/reference/conditional-compilation.html
[_OuterAttribute_]: https://doc.rust-lang.org/reference/attributes.html

## Examples
```rust
/// This is an example struct.
#[cfg_attrs(
    debug_assertions,
    ///
    /// Hello! These are docs that only appear when
    /// debug assertions are active.
)]
struct Example;
```
This can also be written as:
```rust
/// This is an example struct.
#[cfg_attrs(debug_assertions, {
    ///
    /// Hello! These are docs that only appear when
    /// debug assertions are active.
})]
struct Example;
```
Either of these will expand to the following usage of [`#[cfg_attr(...)]`][cfg_attr]:
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
little advantage over using [`#[cfg_attr(...)]`][cfg_attr] in these cases:
```rust
#[cfg_attrs(
    feature = "magic",
    #[sparkles]
    #[crackles]
)]
fn bewitched() {}
```
With that example being equivalent to:
```rust
#[cfg_attr(feature = "magic", sparkles, crackles)]
fn bewitched() {}
```

[cfg_attr]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
