// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use syn::parse::{ParseBuffer, ParseStream};
use syn::token;

/// More flexible alternative to the [`braced!`] macro.
///
/// # Examples
/// Instead of doing this (with no option to handle the error condition):
/// ```
/// # use syn::braced;
/// #
/// # struct A(syn::token::Brace);
/// #
/// # impl syn::parse::Parse for A {
/// #     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
/// let content;
/// let brace = braced!(content in input);
/// #         Ok(Self(brace))
/// #     }
/// # }
/// #
/// # fn main() {
/// #     let input = quote::quote!({});
/// #     syn::parse2::<A>(input).unwrap();
/// # }
/// ```
/// You can instead do this:
/// ```
/// # struct A(syn::token::Brace);
/// #
/// # fn braced(input: syn::parse::ParseStream) -> syn::Result<(syn::token::Brace, syn::parse::ParseBuffer)> {
/// #     syn::__private::parse_braces(input).map(|braces| (braces.token, braces.content))
/// # }
/// #
/// # impl syn::parse::Parse for A {
/// #     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
/// let (brace, content) = braced(input)?;
/// #         Ok(Self(brace))
/// #     }
/// # }
/// #
/// # fn main() {
/// #     let input = quote::quote!({});
/// #     syn::parse2::<A>(input).unwrap();
/// # }
/// ```
///
/// [braced!]: syn::braced!
pub fn braced(input: ParseStream) -> syn::Result<(token::Brace, ParseBuffer)> {
	syn::__private::parse_braces(input).map(|braces| (braces.token, braces.content))
}
