use syn::{
    braced, parenthesized, parse,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, Expr, ExprCall, ExprPath, ExprStruct, Field, FieldValue, Fields, FieldsNamed,
    FieldsUnnamed, Ident, Path, PathSegment, Token, Type, Variant, Visibility,
};

#[derive(Debug, Clone, PartialEq)]
pub struct VariantWithValue {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub fields: MultipleFieldsWithValues,
    pub discriminant: Option<(Token![=], Expr)>,
}

impl VariantWithValue {
    pub fn into_syn_variant(self) -> syn::Variant {
        self.into()
    }
    pub fn into_syn_expr_with_prefix(self, mut prefix: Path) -> syn::Expr {
        prefix.segments.push(PathSegment::from(self.ident));
        let path = prefix;
        match self.fields {
            MultipleFieldsWithValues::Named(MultipleFieldsWithValueNamed {
                brace_token: _,
                fields,
            }) => Expr::Struct(ExprStruct {
                attrs: vec![],
                path,
                brace_token: Default::default(),
                fields: fields
                    .into_iter()
                    .map(
                        |FieldWithValueNamed {
                             ident,
                             colon_token,
                             expr,
                             ..
                         }| FieldValue {
                            attrs: vec![],
                            member: syn::Member::Named(ident),
                            colon_token: Some(colon_token),
                            expr,
                        },
                    )
                    .collect(),
                dot2_token: None,
                rest: None,
                qself: None,
            }),
            MultipleFieldsWithValues::Unnamed(MultipleFieldsWithValuesUnnamed {
                paren_token: _,
                fields,
            }) => Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::from(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path,
                })),
                paren_token: Default::default(),
                args: fields
                    .into_iter()
                    .map(|FieldWithValueUnnamed { expr, .. }| expr)
                    .collect(),
            }),
            MultipleFieldsWithValues::Unit => Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path,
            }),
        }
    }
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
    Unnamed(MultipleFieldsWithValuesUnnamed),
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
            fields: content.parse_terminated(FieldWithValueNamed::parse, Token!(,))?,
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
            mutability: syn::FieldMutability::None,
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
pub struct MultipleFieldsWithValuesUnnamed {
    pub paren_token: token::Paren,
    pub fields: Punctuated<FieldWithValueUnnamed, Token![,]>,
}

impl From<MultipleFieldsWithValuesUnnamed> for FieldsUnnamed {
    fn from(value: MultipleFieldsWithValuesUnnamed) -> Self {
        Self {
            paren_token: value.paren_token,
            unnamed: value.fields.into_iter().map(Field::from).collect(),
        }
    }
}

impl Parse for MultipleFieldsWithValuesUnnamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            fields: content.parse_terminated(FieldWithValueUnnamed::parse, Token!(,))?,
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
            mutability: syn::FieldMutability::None,
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

    use crate::{ident, path};

    use super::*;
    use pretty_assertions::assert_eq;
    use proc_macro2::Span;
    use quote::quote;
    use syn::LitInt;

    fn test_parse<T>(tokens: proc_macro2::TokenStream, expected: T)
    where
        T: syn::parse::Parse + PartialEq + fmt::Debug,
    {
        let actual = syn::parse2::<T>(tokens).expect("couldn't parse tokens");
        assert_eq!(expected, actual);
    }

    fn type_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> Type {
        Type::Path(syn::TypePath {
            qself: None,
            path: path(segments),
        })
    }

    fn test_use(
        call_site: proc_macro2::TokenStream,
        expected_construction: proc_macro2::TokenStream,
        expected_definition: proc_macro2::TokenStream,
    ) {
        let call_site = syn::parse2::<VariantWithValue>(call_site).expect("invalid call_site");

        let expected_construction =
            syn::parse2::<Expr>(expected_construction).expect("invalid expected_construction");
        assert_eq!(
            expected_construction,
            call_site.clone().into_syn_expr_with_prefix(Path {
                leading_colon: None,
                segments: Punctuated::new()
            })
        );

        let expected_definition =
            syn::parse2::<Variant>(expected_definition).expect("invalid expected_definition");
        assert_eq!(expected_definition, call_site.into_syn_variant());
    }

    fn lit_int(s: &str) -> Expr {
        Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Int(LitInt::new(s, Span::call_site())),
        })
    }

    #[test]
    fn parse_unit_variant() {
        test_parse(
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
    fn parse_unit_variant_with_discriminant() {
        test_parse(
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
    fn parse_named_variant() {
        test_parse(
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
                        ty: type_path(["usize"]),
                        eq_token: Default::default(),
                        expr: lit_int("1"),
                    }]),
                }),
                discriminant: None,
            },
        );
    }

    #[test]
    fn parse_unnamed_variant() {
        test_parse(
            quote!(Foo(usize = 1)),
            VariantWithValue {
                attrs: vec![],
                ident: ident("Foo"),
                fields: MultipleFieldsWithValues::Unnamed(MultipleFieldsWithValuesUnnamed {
                    paren_token: Default::default(),
                    fields: Punctuated::from_iter([FieldWithValueUnnamed {
                        attrs: vec![],
                        ty: type_path(["usize"]),
                        eq_token: Default::default(),
                        expr: lit_int("1"),
                    }]),
                }),
                discriminant: None,
            },
        )
    }

    #[test]
    fn use_unit_variant() {
        test_use(quote!(Foo), quote!(Foo), quote!(Foo))
    }
    #[test]
    fn use_named_variant() {
        test_use(
            quote!(Foo {
                bar: usize = 1,
                baz: char = 'a'
            }),
            quote!(Foo { bar: 1, baz: 'a' }),
            quote!(Foo {
                bar: usize,
                baz: char
            }),
        )
    }
    #[test]
    fn use_unnamed_variant() {
        test_use(
            quote!(Foo(usize = 1, char = 'a')),
            quote!(Foo(1, 'a')),
            quote!(Foo(usize, char)),
        )
    }
}
