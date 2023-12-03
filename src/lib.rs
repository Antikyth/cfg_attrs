// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};

use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::{Pair, Punctuated};
use syn::spanned::Spanned;
use syn::{
	parse_macro_input, token, Attribute, Error, Field, Fields, FieldsNamed, Item, Meta, Path, Token, TraitItem,
	WhereClause,
};

#[doc = include_str!("../docs.md")]
#[proc_macro_attribute]
pub fn cfg_attrs(attr: TokenStream, item: TokenStream) -> TokenStream {
	let cfg_attrs_error = if attr.is_empty() {
		None
	} else {
		Some(Error::new(Span::call_site(), "unexpected token in attribute").into_compile_error())
	};

	let item = to_tokens(parse_macro_input!(item as Item));

	let tokens = quote! {
		#cfg_attrs_error
		#item
	};

	tokens.into()
}

enum Attr {
	Configure {
		hash: Token![#],
		square_bracket: token::Bracket,
		path: Path,
		meta: ConfigureMeta,
	},

	Other(Attribute),
}

struct ConfigureMeta {
	condition: Meta,
	comma: Token![,],
	attrs: Punctuated<Attr, Token![,]>,
}

fn to_tokens(item: Item) -> TokenStream2 {
	let mut tokens = TokenStream2::new();

	match item {
		Item::Const(r#const) => {
			attrs_to_tokens(r#const.attrs, &mut tokens);

			let (impl_generics, _, where_clause) = r#const.generics.split_for_impl();

			r#const.vis.to_tokens(&mut tokens);
			r#const.const_token.to_tokens(&mut tokens);
			r#const.ident.to_tokens(&mut tokens);

			impl_generics.to_tokens(&mut tokens);

			r#const.colon_token.to_tokens(&mut tokens);
			r#const.ty.to_tokens(&mut tokens);
			r#const.eq_token.to_tokens(&mut tokens);
			r#const.expr.to_tokens(&mut tokens);

			where_clause.to_tokens(&mut tokens);

			r#const.semi_token.to_tokens(&mut tokens);
		},

		Item::Enum(r#enum) => {
			attrs_to_tokens(r#enum.attrs, &mut tokens);

			let (impl_generics, _, where_clause) = r#enum.generics.split_for_impl();

			r#enum.vis.to_tokens(&mut tokens);
			r#enum.enum_token.to_tokens(&mut tokens);
			r#enum.ident.to_tokens(&mut tokens);

			impl_generics.to_tokens(&mut tokens);
			where_clause.to_tokens(&mut tokens);

			r#enum.brace_token.surround(&mut tokens, |tokens| {
				for pair in r#enum.variants.into_pairs() {
					let (variant, comma) = match pair {
						Pair::Punctuated(variant, comma) => (variant, Some(comma)),
						Pair::End(variant) => (variant, None),
					};

					attrs_to_tokens(variant.attrs, tokens);

					variant.ident.to_tokens(tokens);
					if let Some((eq, discrim)) = &variant.discriminant {
						eq.to_tokens(tokens);
						discrim.to_tokens(tokens);
					};

					fields_to_tokens(variant.fields, None, tokens);

					comma.to_tokens(tokens);
				}
			});
		},

		Item::ExternCrate(r#extern) => {
			attrs_to_tokens(r#extern.attrs, &mut tokens);

			r#extern.vis.to_tokens(&mut tokens);
			r#extern.extern_token.to_tokens(&mut tokens);
			r#extern.crate_token.to_tokens(&mut tokens);
			r#extern.ident.to_tokens(&mut tokens);
			if let Some((r#as, name)) = r#extern.rename {
				r#as.to_tokens(&mut tokens);
				name.to_tokens(&mut tokens);
			}
			r#extern.semi_token.to_tokens(&mut tokens);
		},

		Item::Fn(r#fn) => {
			attrs_to_tokens(r#fn.attrs, &mut tokens);

			r#fn.vis.to_tokens(&mut tokens);
			r#fn.sig.to_tokens(&mut tokens);
			r#fn.block.to_tokens(&mut tokens);
		},

		Item::Macro(r#macro) => {
			attrs_to_tokens(r#macro.attrs, &mut tokens);

			r#macro.ident.to_tokens(&mut tokens);
			r#macro.mac.to_tokens(&mut tokens);
			r#macro.semi_token.to_tokens(&mut tokens);
		},

		Item::Static(r#static) => {
			attrs_to_tokens(r#static.attrs, &mut tokens);

			r#static.vis.to_tokens(&mut tokens);
			r#static.static_token.to_tokens(&mut tokens);
			r#static.mutability.to_tokens(&mut tokens);
			r#static.ident.to_tokens(&mut tokens);
			r#static.colon_token.to_tokens(&mut tokens);
			r#static.ty.to_tokens(&mut tokens);
			r#static.eq_token.to_tokens(&mut tokens);
			r#static.expr.to_tokens(&mut tokens);
			r#static.semi_token.to_tokens(&mut tokens);
		},

		Item::Struct(r#struct) => {
			attrs_to_tokens(r#struct.attrs, &mut tokens);

			let (impl_generics, _, where_clause) = r#struct.generics.split_for_impl();

			r#struct.vis.to_tokens(&mut tokens);
			r#struct.struct_token.to_tokens(&mut tokens);
			r#struct.ident.to_tokens(&mut tokens);

			impl_generics.to_tokens(&mut tokens);

			fields_to_tokens(r#struct.fields, where_clause, &mut tokens);
			r#struct.semi_token.to_tokens(&mut tokens);
		},

		Item::Trait(r#trait) => {
			attrs_to_tokens(r#trait.attrs, &mut tokens);

			let (impl_generics, _, where_clause) = r#trait.generics.split_for_impl();

			r#trait.vis.to_tokens(&mut tokens);
			r#trait.unsafety.to_tokens(&mut tokens);
			r#trait.auto_token.to_tokens(&mut tokens);
			r#trait.trait_token.to_tokens(&mut tokens);
			r#trait.ident.to_tokens(&mut tokens);

			impl_generics.to_tokens(&mut tokens);

			r#trait.colon_token.to_tokens(&mut tokens);
			r#trait.supertraits.to_tokens(&mut tokens);

			where_clause.to_tokens(&mut tokens);

			r#trait.brace_token.surround(&mut tokens, |tokens| {
				for item in r#trait.items {
					match item {
						TraitItem::Const(r#const) => {
							attrs_to_tokens(r#const.attrs, tokens);

							let (impl_generics, _, where_clause) = r#const.generics.split_for_impl();

							r#const.const_token.to_tokens(tokens);
							r#const.ident.to_tokens(tokens);

							impl_generics.to_tokens(tokens);

							r#const.colon_token.to_tokens(tokens);
							r#const.ty.to_tokens(tokens);
							if let Some((eq, expr)) = &r#const.default {
								eq.to_tokens(tokens);
								expr.to_tokens(tokens);
							}

							where_clause.to_tokens(tokens);

							r#const.semi_token.to_tokens(tokens);
						},

						TraitItem::Fn(r#fn) => {
							attrs_to_tokens(r#fn.attrs, tokens);

							r#fn.sig.to_tokens(tokens);
							r#fn.default.to_tokens(tokens);
							r#fn.semi_token.to_tokens(tokens);
						},

						TraitItem::Macro(r#macro) => {
							attrs_to_tokens(r#macro.attrs, tokens);

							r#macro.mac.to_tokens(tokens);
							r#macro.semi_token.to_tokens(tokens);
						},

						TraitItem::Type(r#type) => {
							attrs_to_tokens(r#type.attrs, tokens);

							let (impl_generics, _, where_clause) = r#type.generics.split_for_impl();

							r#type.type_token.to_tokens(tokens);
							r#type.ident.to_tokens(tokens);

							impl_generics.to_tokens(tokens);

							r#type.colon_token.to_tokens(tokens);
							r#type.bounds.to_tokens(tokens);
							if let Some((eq, r#type)) = &r#type.default {
								eq.to_tokens(tokens);
								r#type.to_tokens(tokens);
							}

							where_clause.to_tokens(tokens);

							r#type.semi_token.to_tokens(tokens);
						},

						TraitItem::Verbatim(token_stream) => token_stream.to_tokens(tokens),

						_ => {},
					}
				}
			});
		},

		Item::TraitAlias(alias) => {
			attrs_to_tokens(alias.attrs, &mut tokens);

			let (impl_generics, _, where_clause) = alias.generics.split_for_impl();

			alias.vis.to_tokens(&mut tokens);
			alias.trait_token.to_tokens(&mut tokens);
			alias.ident.to_tokens(&mut tokens);

			impl_generics.to_tokens(&mut tokens);

			alias.eq_token.to_tokens(&mut tokens);
			alias.bounds.to_tokens(&mut tokens);

			where_clause.to_tokens(&mut tokens);

			alias.semi_token.to_tokens(&mut tokens);
		},

		Item::Type(r#type) => {
			attrs_to_tokens(r#type.attrs, &mut tokens);

			let (impl_generics, _, where_clause) = r#type.generics.split_for_impl();

			r#type.vis.to_tokens(&mut tokens);
			r#type.type_token.to_tokens(&mut tokens);
			r#type.ident.to_tokens(&mut tokens);

			impl_generics.to_tokens(&mut tokens);

			r#type.eq_token.to_tokens(&mut tokens);
			r#type.ty.to_tokens(&mut tokens);

			where_clause.to_tokens(&mut tokens);

			r#type.semi_token.to_tokens(&mut tokens);
		},

		Item::Use(r#use) => {
			attrs_to_tokens(r#use.attrs, &mut tokens);

			r#use.vis.to_tokens(&mut tokens);
			r#use.use_token.to_tokens(&mut tokens);
			r#use.leading_colon.to_tokens(&mut tokens);
			r#use.tree.to_tokens(&mut tokens);
			r#use.semi_token.to_tokens(&mut tokens);
		},

		Item::Verbatim(token_stream) => token_stream.to_tokens(&mut tokens),

		_ => (),
	}

	tokens
}

fn attrs_to_tokens(attrs: Vec<Attribute>, tokens: &mut TokenStream2) {
	for attribute in attrs {
		Attr::try_from(attribute)
			.map_or_else(Error::into_compile_error, ToTokens::into_token_stream)
			.to_tokens(tokens);
	}
}

fn fields_to_tokens(fields: Fields, where_clause: Option<&WhereClause>, tokens: &mut TokenStream2) {
	match fields {
		Fields::Unit => where_clause.to_tokens(tokens),

		Fields::Named(named) => {
			where_clause.to_tokens(tokens);
			fields_named_to_tokens(named, tokens)
		},

		Fields::Unnamed(unnamed) => unnamed.paren_token.surround(tokens, |tokens| {
			for pair in unnamed.unnamed.into_pairs() {
				let (field, comma) = match pair {
					Pair::Punctuated(field, comma) => (field, Some(comma)),
					Pair::End(field) => (field, None),
				};

				field_to_tokens(field, tokens);
				comma.to_tokens(tokens);
			}

			where_clause.to_tokens(tokens);
		}),
	}
}

fn fields_named_to_tokens(fields: FieldsNamed, tokens: &mut TokenStream2) {
	fields.brace_token.surround(tokens, |tokens| {
		for pair in fields.named.into_pairs() {
			let (field, comma) = match pair {
				Pair::Punctuated(field, comma) => (field, Some(comma)),
				Pair::End(field) => (field, None),
			};

			field_to_tokens(field, tokens);
			comma.to_tokens(tokens);
		}
	})
}

fn field_to_tokens(field: Field, tokens: &mut TokenStream2) {
	attrs_to_tokens(field.attrs, tokens);

	field.vis.to_tokens(tokens);
	field.ident.to_tokens(tokens);
	field.colon_token.to_tokens(tokens);
	field.ty.to_tokens(tokens);
}

impl ToTokens for Attr {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		match self {
			Self::Configure {
				hash, square_bracket, ..
			} => {
				hash.to_tokens(tokens);
				square_bracket.surround(tokens, |tokens| self.meta_to_tokens(tokens));
			},

			Self::Other(attribute) => attribute.to_tokens(tokens),
		}
	}
}

impl Attr {
	fn meta_to_tokens(&self, tokens: &mut TokenStream2) {
		match self {
			Self::Configure { path, meta, .. } => {
				let path = quote_spanned!(path.span()=> cfg_attr);
				quote!(#path(#meta)).to_tokens(tokens);
			},

			Self::Other(Attribute { meta, .. }) => meta.to_tokens(tokens),
		}
	}
}

impl ToTokens for ConfigureMeta {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		let attrs = self.attrs.pairs().map(|pair| match pair {
			Pair::Punctuated(attr, comma) => (attr, Some(comma)),
			Pair::End(attr) => (attr, None),
		});

		self.condition.to_tokens(tokens);
		self.comma.to_tokens(tokens);

		for (attr, comma) in attrs {
			attr.meta_to_tokens(tokens);
			comma.to_tokens(tokens);
		}
	}
}

impl TryFrom<Attribute> for Attr {
	type Error = Error;

	fn try_from(attribute: Attribute) -> syn::Result<Self> {
		Ok(if attribute.path().is_ident("configure") {
			let (path, meta) = match attribute.meta {
				Meta::List(list) => (list.path, syn::parse2(list.tokens)?),
				other => {
					return Err(Error::new(
						other.span(),
						"expected attribute arguments in parentheses: `configure(...)`",
					))
				},
			};

			Attr::Configure {
				hash: attribute.pound_token,
				square_bracket: attribute.bracket_token,
				path,
				meta,
			}
		} else {
			Attr::Other(attribute)
		})
	}
}

impl Attr {
	fn parse(input: ParseStream) -> syn::Result<Vec<Self>> {
		let attributes = input.call(Attribute::parse_outer)?;
		let mut attrs = Vec::with_capacity(attributes.len());

		for attribute in attributes {
			attrs.push(attribute.try_into()?);
		}

		Ok(attrs)
	}
}

impl Parse for ConfigureMeta {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			condition: input.parse()?,
			comma: input.parse()?,
			attrs: input
				.parse_terminated(Attr::parse, Token![,])
				.into_iter()
				.flatten()
				.flatten()
				.collect(),
		})
	}
}
