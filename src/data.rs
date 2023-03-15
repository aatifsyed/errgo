use syn::{
    braced, parenthesized, parse,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Token, Type, Variant,
    Visibility,
};

#[derive(Debug, Clone, PartialEq)]
pub struct VariantWithValue {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub fields: MultipleFieldsWithValues,
    pub discriminant: Option<(Token![=], Expr)>,
}

impl From<VariantWithValue> for Variant {
    fn from(value: VariantWithValue) -> Self {
        Self {
            attrs: value.attrs,
            ident: value.ident,
            fields: value.fields.into(),
            discriminant: value.discriminant,
        }
    }
}

impl Parse for VariantWithValue {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let _visibility: Visibility = input.parse()?;
        let ident: Ident = input.parse()?;
        let fields = if input.peek(token::Brace) {
            MultipleFieldsWithValues::Named(input.parse()?)
        } else if input.peek(token::Paren) {
            MultipleFieldsWithValues::Unnamed(input.parse()?)
        } else {
            MultipleFieldsWithValues::Unit
        };
        let discriminant = if input.peek(Token![=]) {
            let eq_token: Token![=] = input.parse()?;
            let discriminant: Expr = input.parse()?;
            Some((eq_token, discriminant))
        } else {
            None
        };
        Ok(Self {
            attrs,
            ident,
            fields,
            discriminant,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MultipleFieldsWithValues {
    Named(MultipleFieldsWithValueNamed),
    Unnamed(MultipleFieldsWithValueUnnamed),
    Unit,
}

impl From<MultipleFieldsWithValues> for Fields {
    fn from(value: MultipleFieldsWithValues) -> Self {
        match value {
            MultipleFieldsWithValues::Named(n) => Self::Named(n.into()),
            MultipleFieldsWithValues::Unnamed(u) => Self::Unnamed(u.into()),
            MultipleFieldsWithValues::Unit => Self::Unit,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultipleFieldsWithValueNamed {
    pub brace_token: token::Brace,
    pub fields: Punctuated<FieldWithValueNamed, Token![,]>,
}

impl From<MultipleFieldsWithValueNamed> for FieldsNamed {
    fn from(value: MultipleFieldsWithValueNamed) -> Self {
        Self {
            brace_token: value.brace_token,
            named: value.fields.into_iter().map(Field::from).collect(),
        }
    }
}

impl Parse for MultipleFieldsWithValueNamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            brace_token: braced!(content in input),
            fields: content.parse_terminated(FieldWithValueNamed::parse)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldWithValueNamed {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Type,
    pub eq_token: Token![=],
    pub expr: Expr,
}

impl From<FieldWithValueNamed> for Field {
    fn from(value: FieldWithValueNamed) -> Self {
        Self {
            attrs: value.attrs,
            vis: Visibility::Inherited,
            ident: Some(value.ident),
            colon_token: Some(value.colon_token),
            ty: value.ty,
        }
    }
}

impl Parse for FieldWithValueNamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultipleFieldsWithValueUnnamed {
    pub paren_token: token::Paren,
    pub fields: Punctuated<FieldWithValueUnnamed, Token![,]>,
}

impl From<MultipleFieldsWithValueUnnamed> for FieldsUnnamed {
    fn from(value: MultipleFieldsWithValueUnnamed) -> Self {
        Self {
            paren_token: value.paren_token,
            unnamed: value.fields.into_iter().map(Field::from).collect(),
        }
    }
}

impl Parse for MultipleFieldsWithValueUnnamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            fields: content.parse_terminated(FieldWithValueUnnamed::parse)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldWithValueUnnamed {
    pub attrs: Vec<Attribute>,
    pub ty: Type,
    pub eq_token: Token![=],
    pub expr: Expr,
}

impl From<FieldWithValueUnnamed> for Field {
    fn from(value: FieldWithValueUnnamed) -> Self {
        Self {
            attrs: value.attrs,
            vis: Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: value.ty,
        }
    }
}

impl Parse for FieldWithValueUnnamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ty: input.parse()?,
            eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use super::*;
    use pretty_assertions::assert_eq;
    use proc_macro2::Span;
    use quote::quote;
    use syn::{LitInt, PathSegment};

    fn do_test<T>(tokens: proc_macro2::TokenStream, expected: T)
    where
        T: syn::parse::Parse + PartialEq + fmt::Debug,
    {
        let actual = syn::parse2::<T>(tokens).expect("couldn't parse tokens");
        assert_eq!(expected, actual);
    }

    fn ident(s: &str) -> Ident {
        Ident::new(s, Span::call_site())
    }

    fn lit_int(s: &str) -> Expr {
        Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Int(LitInt::new(s, Span::call_site())),
        })
    }

    fn path<'a>(segments: impl IntoIterator<Item = &'a str>) -> Type {
        Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: segments
                    .into_iter()
                    .map(|segment| PathSegment::from(ident(segment)))
                    .collect(),
            },
        })
    }

    #[test]
    fn unit_variant() {
        do_test(
            quote! {Foo},
            VariantWithValue {
                attrs: vec![],
                ident: ident("Foo"),
                fields: MultipleFieldsWithValues::Unit,
                discriminant: None,
            },
        )
    }

    #[test]
    fn unit_variant_with_discriminant() {
        do_test(
            quote! {Foo = 1},
            VariantWithValue {
                attrs: vec![],
                ident: ident("Foo"),
                fields: MultipleFieldsWithValues::Unit,
                discriminant: Some((Default::default(), lit_int("1"))),
            },
        )
    }

    #[test]
    fn named_variant() {
        do_test(
            quote!(Foo { bar: usize = 1 }),
            VariantWithValue {
                attrs: vec![],
                ident: ident("Foo"),
                fields: MultipleFieldsWithValues::Named(MultipleFieldsWithValueNamed {
                    brace_token: Default::default(),
                    fields: Punctuated::from_iter([FieldWithValueNamed {
                        attrs: vec![],
                        ident: ident("bar"),
                        colon_token: Default::default(),
                        ty: path(["usize"]),
                        eq_token: Default::default(),
                        expr: lit_int("1"),
                    }]),
                }),
                discriminant: None,
            },
        )
    }

    #[test]
    fn unnamed_variant() {
        do_test(
            quote!(Foo(usize = 1)),
            VariantWithValue {
                attrs: vec![],
                ident: ident("Foo"),
                fields: MultipleFieldsWithValues::Unnamed(MultipleFieldsWithValueUnnamed {
                    paren_token: Default::default(),
                    fields: Punctuated::from_iter([FieldWithValueUnnamed {
                        attrs: vec![],
                        ty: path(["usize"]),
                        eq_token: Default::default(),
                        expr: lit_int("1"),
                    }]),
                }),
                discriminant: None,
            },
        )
    }
}
