// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, token, Attribute, Error, Meta, Path, Token};

enum Attr {
	/// A `#[configure(...)]` attribute.
	Configure {
		/// `#`
		hash: Token![#],
		/// `[` ... `]`
		square_bracket: token::Bracket,
		/// `configure`
		_path: Path,

		meta: ConfigureMeta,
	},

	/// Not a `#[configure(...)]` attribute.
	Other(Attribute),
}

/// The `$(#[$meta:meta])*` part of `#[cfg_attrs { $(#[$meta:meta])* }]`.
struct CfgAttrsMeta(
	/// `$(#[$meta:meta])*`
	Vec<Attr>,
);

/// The `...` part of `#[configure(...)]`.
struct ConfigureMeta {
	/// ```
	/// # macro_rules! _example {
	/// #     (
	/// $condition:meta
	/// #     ) => {};
	/// # }
	/// ```
	condition: Meta,
	/// ```
	/// # macro_rules! _example {
	/// #     (
	/// ,
	/// #     ) => {};
	/// # }
	/// ```
	comma: Token![,],
	/// ```
	/// # macro_rules! _example {
	/// #     (
	/// $($($(#[$meta:meta])+),+$(,)?)?
	/// #     ) => {};
	/// # }
	/// ```
	metas: Punctuated<Meta, Token![,]>,
}

fn parse_metas(input: ParseStream) -> syn::Result<Punctuated<Meta, Token![,]>> {
	Ok(input
		.parse_terminated(Attribute::parse_outer, Token![,])?
		.into_iter()
		.flatten()
		.map(|attribute| attribute.meta)
		.collect())
}

impl Parse for CfgAttrsMeta {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attributes = input.call(Attribute::parse_outer)?;
		let mut attrs = Vec::with_capacity(attributes.len());

		for attribute in attributes {
			let attr = if attribute.path().is_ident("configure") {
				let (tokens, _path) = match attribute.meta {
					Meta::List(list) => (list.tokens, list.path),

					meta => {
						return Err(Error::new(
							meta.span(),
							"expected attribute arguments in parentheses: configure(...)",
						))
					},
				};

				Attr::Configure {
					hash: attribute.pound_token,
					square_bracket: attribute.bracket_token,
					_path,

					meta: syn::parse2(tokens)?,
				}
			} else {
				Attr::Other(attribute)
			};

			attrs.push(attr);
		}

		Ok(Self(attrs))
	}
}

impl Parse for ConfigureMeta {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			condition: input.parse()?,
			comma: input.parse()?,
			metas: input.call(parse_metas)?,
		})
	}
}

impl ToTokens for Attr {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		match self {
			Self::Configure {
				hash,
				square_bracket,
				meta,
				..
			} => {
				hash.to_tokens(tokens);
				square_bracket.surround(tokens, |tokens| quote!(cfg_attr(#meta)).to_tokens(tokens));
			},

			Self::Other(attr) => attr.to_tokens(tokens),
		}
	}
}

impl ToTokens for CfgAttrsMeta {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		let Self(attrs) = self;

		for attr in attrs {
			attr.to_tokens(tokens);
		}
	}
}

impl ToTokens for ConfigureMeta {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		// `$condition:meta`
		self.condition.to_tokens(tokens);
		// `,`
		self.comma.to_tokens(tokens);
		// `$($($(#[$meta:meta])+),+$(,)?)?`
		self.metas.to_tokens(tokens);
	}
}

/// Provides an alternative syntax to [`#[cfg_attr(...)]`] that is easier to use with doc
/// comments.
///
/// > <sup>Syntax</sup> \
/// > _CfgAttrsAttribute_ : \
/// > &nbsp;&nbsp;`cfg_attrs` _CfgAttrs_
/// >
/// > _CfgAttrs_ : \
/// > &nbsp;&nbsp;( `{` _Attributes_ `}` ) | _ConfiguredAttrs_
/// >
/// > _ConfiguredAttrs_ : \
/// > &nbsp;&nbsp;`(` [_ConfigurationPredicate_] `,` _Attributes_ `)`
/// >
/// > _Attributes_ : \
/// > &nbsp;&nbsp;[_OuterAttribute_]<sup>\*</sup> ( `,` [_OuterAttribute_]<sup>\*</sup> )<sup>\*</sup> `,`<sup>?</sup>
///
/// [_ConfigurationPredicate_]: https://doc.rust-lang.org/reference/conditional-compilation.html
/// [_OuterAttribute_]: https://doc.rust-lang.org/reference/attributes.html
///
/// # Usage
/// `#[cfg_attrs { ... }]` should surround all other attributes on the item. A
/// `#[configure(<condition>, <attributes>)]` helper attribute is provided within.
///
/// The syntax of that `#[configure(...)]` attribute is much like [`#[cfg_attr(...)]`], except the
/// configured attributes use full attribute syntax. The advantage of this is that doc comments,
/// which expand to `#[doc = "..."]` attributes, can be used in the `#[configure(...)]` syntax.
///
/// <div class="warning">
///
/// All of an item's doc comments should be placed within the `#[cfg_attrs { ... }]` attribute, even
/// if they are not being configured. Additionally, the `#[cfg_attrs { ... }]` attribute should only
/// appear once per item; `#[configure(...)]` can be used multiple times within it if you want
/// multiple usages.
///
/// </div>
///
/// These restrictions are as a result of how proc-macro attributes work: they are expanded
/// separately to other attributes, so their position among other attributes is lost. While you
/// might put a `#[cfg_attrs { ... }]` attribute that configures doc comments between two
/// non-configured doc comments, that isn't where it will be expanded to, so the documentation will
/// be out of order.
///
/// # Examples
/// ```
/// # use cfg_attrs::cfg_attrs;
/// #
/// #[cfg_attrs {
///     /// This is an example struct.
///     #[configure(
///         debug_assertions,
///         ///
///         /// Hello! These are docs that only appear when
///         /// debug assertions are active.
///     )]
/// }]
/// struct Example;
/// ```
/// This will expand to the following usage of [`#[cfg_attr(...)]`]:
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
/// no real benefit to doing this:
/// ```
/// # use cfg_attrs::cfg_attrs;
/// #
/// #[cfg_attrs {
///     #[configure(
///         feature = "magic",
///         #[sparkles]
///         #[crackles]
///     )]
/// }]
/// fn bewitched() {}
/// ```
/// With that example being equivalent to:
/// ```
/// #[cfg_attr(feature = "magic", sparkles, crackles)]
/// fn bewitched() {}
/// ```
///
/// [`#[cfg_attr(...)]`]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
#[proc_macro_attribute]
pub fn cfg_attrs(attr: TokenStream, item: TokenStream) -> TokenStream {
	let cfg_attrs = parse_macro_input!(attr as CfgAttrsMeta);
	let item: proc_macro2::TokenStream = item.into();

	let tokens = quote! {
		#cfg_attrs
		#item
	};

	tokens.into()
}
