use quote::ToTokens;
use syn::{
    meta::ParseNestedMeta,
    parenthesized,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Attribute, Path, Token, Visibility,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub derives: Option<Vec<Path>>,
    pub attributes: Option<Vec<Attribute>>,
    pub visibility: Option<Visibility>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = Self::default();
        if !input.is_empty() {
            syn::meta::parser(|stage| config.parse_stage(stage)).parse2(input.parse()?)?;
        }
        Ok(config)
    }
}

impl Config {
    fn parse_stage(&mut self, stage: ParseNestedMeta) -> syn::Result<()> {
        if stage.path.is_ident("derive") {
            let content;
            parenthesized!(content in stage.input);
            let derives = Punctuated::<Path, Token![,]>::parse_terminated(&content)?;
            if derives.is_empty() {
                return Err(stage.error("`derive` cannot be empty"));
            }
            if self.derives.is_some() {
                return Err(stage.error("`derive` specified more than once"));
            }
            self.derives = Some(derives.into_iter().collect());
        } else if stage.path.is_ident("attributes")
            || stage.path.is_ident("attrs")
            || stage.path.is_ident("attr")
        {
            let content;
            parenthesized!(content in stage.input);
            let attributes = Punctuated::<_, Token![,]>::parse_terminated_with(
                &content,
                Attribute::parse_outer,
            )?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
            if attributes.is_empty() {
                return Err(stage.error("`attributes` cannot be empty"));
            }
            if self.attributes.is_some() {
                return Err(stage.error("`attributes` specified more than once"));
            }
            self.attributes = Some(attributes);
        } else if stage.path.is_ident("visibility") || stage.path.is_ident("vis") {
            let content;
            parenthesized!(content in stage.input);
            if self.visibility.is_some() {
                return Err(stage.error("`visibility` specified more than once"));
            }
            self.visibility = Some(content.parse()?);
        } else {
            return Err(stage.error(format!(
                "unexpected argument `{}`, expected `derive` or `attributes` or `visibility`",
                stage.path.to_token_stream()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{path, test_utils::test_parse};
    use proc_macro2::TokenStream;
    use quote::quote;

    fn attributes(tokens: TokenStream) -> Vec<Attribute> {
        Attribute::parse_outer
            .parse2(tokens)
            .expect("unable to parse tokens as attribute")
    }

    #[test]
    fn parse_derives() {
        test_parse(
            quote! {
                derive(Hello, path::to::Goodbye)
            },
            Config {
                derives: Some(vec![path(["Hello"]), path(["path", "to", "Goodbye"])]),
                attributes: None,
                visibility: None,
            },
        );
    }

    #[test]
    fn parse_attributes() {
        test_parse(
            quote! {
                attributes(#[error("foo")], #[repr(u8)])
            },
            Config {
                derives: None,
                attributes: Some(attributes(quote! {
                    #[error("foo")]
                    #[repr(u8)]
                })),
                visibility: None,
            },
        );
    }

    #[test]
    fn parse_visibility() {}

    #[test]
    fn parse_all() {
        test_parse(
            quote! {
                attributes(#[error("foo")], #[repr(u8)]),
                derive(Hello, path::to::Goodbye)
            },
            Config {
                derives: Some(vec![path(["Hello"]), path(["path", "to", "Goodbye"])]),
                attributes: Some(attributes(quote! {
                    #[error("foo")]
                    #[repr(u8)]
                })),
                visibility: None,
            },
        );
    }
}
