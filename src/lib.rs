// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
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
	attrs: Vec<Attr>,
}

fn parse_attrs(input: ParseStream) -> syn::Result<Vec<Attr>> {
	Ok(input
		.parse_terminated(Attr::parse, Token![,])?
		.into_iter()
		.flatten()
		.collect())
}

impl Attr {
	fn parse(input: ParseStream) -> syn::Result<Vec<Self>> {
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

		Ok(attrs)
	}
}

impl Parse for CfgAttrsMeta {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self(input.call(Attr::parse)?))
	}
}

impl Parse for ConfigureMeta {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			condition: input.parse()?,
			comma: input.parse()?,
			attrs: input.call(parse_attrs)?,
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
		self.condition.to_tokens(tokens);
		self.comma.to_tokens(tokens);

		let attr = &self.attrs;
		quote!(::cfg_attrs::cfg_attrs { #(#attr)* }).to_tokens(tokens);
	}
}

#[doc = include_str!("../docs.md")]
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
