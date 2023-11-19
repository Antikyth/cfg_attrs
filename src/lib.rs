// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod util;

use proc_macro::TokenStream;

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, token, Attribute, Meta, Token};

use util::braced;

struct CfgAttrsAttribute {
	condition: Meta,
	comma: Token![,],

	_brace: Option<token::Brace>,
	metas: Vec<Meta>,
}

fn parse_metas(input: ParseStream) -> syn::Result<Vec<Meta>> {
	Ok(input
		.parse_terminated(Attribute::parse_outer, Token![,])?
		.into_iter()
		.flatten()
		.map(|attribute| attribute.meta)
		.collect())
}

impl Parse for CfgAttrsAttribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let condition = input.parse()?;
		let comma = input.parse()?;

		let (brace, content) = match braced(input) {
			Ok((brace, content)) => (Some(brace), Some(content)),
			Err(_) => (None, None),
		};
		let metas = parse_metas(content.as_ref().unwrap_or(input))?;

		Ok(Self {
			condition,
			comma,

			_brace: brace,
			metas,
		})
	}
}

/// Provides an alternative syntax to [`#[cfg_attr(...)]`][cfg_attr] that is easier to use with doc
/// comments.
///
/// > <sup>Syntax</sup> \
/// > _CfgAttrsAttribute_ : \
/// > `cfg_attrs` `(` [_ConfigurationPredicate_] `,` _CfgAttrs_ `)`
/// >
/// > _CfgAttrs_ : \
/// > `{` _Attributes_ `}` | _Attributes_
/// >
/// > _Attributes_ : \
/// > [_OuterAttribute_]<sup>\*</sup> ( `,` [_OuterAttribute_]<sup>\*</sup> )<sup>\*</sup> `,`<sup>?</sup>
///
/// [_ConfigurationPredicate_]: https://doc.rust-lang.org/reference/conditional-compilation.html
/// [_OuterAttribute_]: https://doc.rust-lang.org/reference/attributes.html
///
/// # Examples
/// ```
/// # use cfg_attrs::cfg_attrs;
/// #
/// /// This is an example struct.
/// #[cfg_attrs(
///     debug_assertions,
///     ///
///     /// Hello! These are docs that only appear when
///     /// debug assertions are active.
/// )]
/// struct Example;
/// ```
/// This can also be written as:
/// ```
/// # use cfg_attrs::cfg_attrs;
/// #
/// /// This is an example struct.
/// #[cfg_attrs(debug_assertions, {
///     ///
///     /// Hello! These are docs that only appear when
///     /// debug assertions are active.
/// })]
/// struct Example;
/// ```
/// Either of these will expand to the following usage of [`#[cfg_attr(...)]`][cfg_attr]:
/// ```
/// /// This is an example struct.
/// #[cfg_attr(
///     debug_assertions,
///     doc = "",
///     doc = " Hello! These are docs that only appear when",
///     doc = " debug assertions are active."
/// )]
/// struct Example;
/// ```
/// Which, if debug assertions are active, would be expanded to:
/// ```
/// /// This is an example struct.
/// ///
/// /// Hello! These are docs that only appear when
/// /// debug assertions are active.
/// struct Example;
/// ```
///
/// `#[cfg_attrs(...)]` may also be used with attributes other than doc comments, though there is
/// little advantage over using [`#[cfg_attr(...)]`][cfg_attr] in these cases:
/// ```
/// # use cfg_attrs::cfg_attrs;
/// #
/// #[cfg_attrs(
///     feature = "magic",
///     #[sparkles]
///     #[crackles]
/// )]
/// fn bewitched() {}
/// ```
/// With that example being equivalent to:
/// ```
/// #[cfg_attr(feature = "magic", sparkles, crackles)]
/// fn bewitched() {}
/// ```
///
/// [cfg_attr]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
#[proc_macro_attribute]
pub fn cfg_attrs(attr: TokenStream, item: TokenStream) -> TokenStream {
	let CfgAttrsAttribute {
		condition,
		comma,
		metas: meta,
		..
	} = parse_macro_input!(attr as CfgAttrsAttribute);
	let item: proc_macro2::TokenStream = item.into();

	let tokens = quote! {
		#[cfg_attr(#condition #comma #(#meta),*)]
		#item
	};

	tokens.into()
}
