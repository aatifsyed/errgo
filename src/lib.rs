//! Generate `enum` error variants inline.
//!
//! A slightly type-safer take on [anyhow], where each ad-hoc error is handleable by the caller.
//! Designed to play nice with other crates like [strum] or [thiserror].
//!
//! This crate was written to aid wrapping C APIs - transforming e.g error codes to handleable messages.
//! It shouldn't really be used for library api entry points - a well-considered top-level error type is likely to be both more readable and forward compatible.
//! Consider reading [Study of `std::io::Error`](https://matklad.github.io/2020/10/15/study-of-std-io-error.html) or simply making all generated structs `pub(crate)`.
//!
//! ```
//! use errgo::errgo;
//!
//! #[errgo]
//! fn shave_yaks(
//!     num_yaks: usize,
//!     empty_buckets: usize,
//!     num_razors: usize,
//! ) -> Result<(), ShaveYaksError> {
//!     if num_razors == 0 {
//!         return Err(err!(NotEnoughRazors));
//!     }
//!     if num_yaks > empty_buckets {
//!         return Err(err!(NotEnoughBuckets {
//!             got: usize = empty_buckets,
//!             required: usize = num_yaks,
//!         }));
//!     }
//!     Ok(())
//! }
//! ```
//! Under the hood, a struct like this is generated:
//! ```
//! enum ShaveYaksError { // name and visibility are taken from function return type and visibility
//!     NotEnoughRazors,
//!     NotEnoughBuckets {
//!         got: usize,
//!         required: usize,
//!     }
//! }
//! ```
//! Note that the struct definition is placed just above the function body, meaning that you can't use [`macro@errgo`] on functions in `impl` blocks - you'll have to move the function body to an outer scope, and call it in the impl block.
//!
//!
//! Importantly, you can derive on the generated struct, _and_ passthrough attributes, allowing you to use crates like [thiserror] or [strum].
//! See the [`macro@errgo`] documentation for other arguments accepted by the macro.
//! ```
//! # use errgo::errgo;
//!
//! #[errgo(derive(Debug, thiserror::Error))]
//! fn shave_yaks(
//!     num_yaks: usize,
//!     empty_buckets: usize,
//!     num_razors: usize,
//! ) -> Result<(), ShaveYaksError> {
//!     if num_razors == 0 {
//!         return Err(err!(
//!             #[error("not enough razors!")]
//!             NotEnoughRazors
//!         ));
//!     }
//!     if num_yaks > empty_buckets {
//!         return Err(err!(
//!             #[error("not enough buckets - needed {required}")]
//!             NotEnoughBuckets {
//!                 got: usize = empty_buckets,
//!                 required: usize = num_yaks,
//!             }
//!         ));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Which generates the following:
//! ```
//! #[derive(Debug, thiserror::Error)]
//! enum ShaveYaksError {
//!     #[error("not enough razors!")]
//!     NotEnoughRazors,
//!     #[error("not enough buckets - needed {required}")]
//!     NotEnoughBuckets {
//!         got: usize,
//!         required: usize,
//!     }
//! }
//! ```
//! And `err!` macro invocations are replaced with struct instantiations - no matter where they are in the function body!
//!
//! If you need to reuse the same variant within a function, just use the normal construction syntax:
//! ```
//! # use errgo::errgo;
//! # use std::io;
//! # fn fallible_op() -> Result<(), io::Error> { todo!() }
//! #[errgo]
//! fn foo() -> Result<(), FooError> {
//!     fallible_op().map_err(|e| err!(IoError(io::Error = e)));
//!     Err(FooError::IoError(todo!()))
//! }
//! ```
//!
//! [anyhow]: https://docs.rs/anyhow
//! [thiserror]: https://docs.rs/thiserror
//! [strum]: https://docs.rs/strum

use config::Config;
use data::VariantWithValue;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{emit_error, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse2, parse_macro_input, visit_mut::VisitMut, AngleBracketedGenericArguments,
    GenericArgument, ItemFn, Path, PathArguments, PathSegment, ReturnType, TypePath,
};

mod config;
mod data;

/// See [module documentation](index.html) for general usage.
///
/// # `err!` construction
/// Instances of `err!` will be parsed like so:
/// ```
/// # #[errgo::errgo]
/// # fn foo() -> Result<(), FooError> {
/// err!(Unity);                        // A unit enum variant
/// err!(Tuply(usize = 1, char = 'a')); // A tuple enum variant
/// err!(Structy {                      // A struct enum variant
///         u: usize = 1,
///         c: char = 'a',
/// });
/// # Ok(())
/// # }
/// ```
/// # Arguments
/// `derive` arguments are passed through to the generated struct.
/// ```
/// # use errgo::errgo;
/// #[errgo(derive(Debug, Clone, Copy))]
/// # fn foo() -> Result<(), FooError> { Ok(()) }
/// ```
///
/// `attributes` arguments are passed through to the top of the generated struct
/// ```
/// # use errgo::errgo;
/// #[errgo(attributes(
///     #[must_use = "maybe you missed something!"]
///     #[repr(u8)]
/// ))]
/// # fn foo() -> Result<(), FooError> { Ok(()) }
/// ```
/// `visibility` can be used to override the generated struct's visibility.
/// ```
/// # use errgo::errgo;
/// #[errgo(visibility(pub))]
/// # fn foo() -> Result<(), FooError> { Ok(()) }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn errgo(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse our inputs
    let config = parse_macro_input!(attr as Config);
    let mut item = parse_macro_input!(item as ItemFn);

    let Some(error_name) = get_struct_name_from_return_type(&item.sig.output) else {
        emit_error!(
            item.sig,
            "unsupported return type - function must return a `Result<_, SomeConcreteErr>`"
        );
        return quote!(#item).into();
    };
    let error_vis = config.visibility.unwrap_or_else(|| item.vis.clone());

    // Make the changes to the syntax tree, and collect the error variants
    let mut visitor = ErrAsYouGoVisitor::new(error_name.clone());
    visitor.visit_item_fn_mut(&mut item);

    for (src, e) in visitor.collection_errors {
        emit_error!(src, "{}", e)
    }

    // Assemble our output
    let variants = visitor.variants;
    let derives = match config.derives {
        Some(derives) => quote!(#[derive(
            #(#derives),*
        )]),
        None => quote!(),
    };

    quote! {
        #derives
        #error_vis enum #error_name {
            #(#variants),*
        }

        #item
    }
    .into()
}

fn get_struct_name_from_return_type(return_type: &ReturnType) -> Option<Ident> {
    if let ReturnType::Type(_, ty) = return_type {
        if let syn::Type::Path(TypePath {
            qself: None,
            path: Path { ref segments, .. },
        }) = **ty
        {
            if let Some(PathSegment {
                ident,
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            }) = segments.last()
            {
                if ident == "Result" && args.len() == 2 {
                    if let Some(GenericArgument::Type(syn::Type::Path(TypePath {
                        qself: None,
                        path:
                            Path {
                                segments,
                                leading_colon: None,
                            },
                    }))) = args.into_iter().nth(1)
                    {
                        if segments.len() == 1 {
                            let PathSegment { ident, arguments } = &segments[0];
                            if arguments.is_empty() {
                                return Some(ident.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Implementation detail
// Allows use to swap the macro in-place in our visitor.
#[doc(hidden)]
#[proc_macro]
#[proc_macro_error]
pub fn __nothing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}

struct ErrAsYouGoVisitor {
    error_name: Ident,
    variants: Vec<syn::Variant>,
    collection_errors: Vec<(TokenStream, syn::Error)>,
}

impl ErrAsYouGoVisitor {
    fn new(error_name: Ident) -> Self {
        Self {
            error_name,
            variants: Vec::new(),
            collection_errors: Vec::new(),
        }
    }
}

impl syn::visit_mut::VisitMut for ErrAsYouGoVisitor {
    fn visit_macro_mut(&mut self, i: &mut syn::Macro) {
        if i.path.is_ident("err") || i.path.is_ident("errgo") {
            match parse2::<VariantWithValue>(i.tokens.clone()) {
                Ok(variant_with_value) => {
                    self.variants
                        .push(variant_with_value.clone().into_syn_variant());
                    i.path = path(["errgo", "__nothing"]);
                    i.tokens = variant_with_value
                        .into_syn_expr_with_prefix(Path::from(self.error_name.clone()))
                        .into_token_stream();
                }
                Err(e) => self.collection_errors.push((i.tokens.clone(), e)),
            }
        }
    }
}

fn path<'a>(segments: impl IntoIterator<Item = &'a str>) -> Path {
    syn::Path {
        leading_colon: None,
        segments: segments
            .into_iter()
            .map(|segment| PathSegment::from(ident(segment)))
            .collect(),
    }
}

fn ident(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

#[cfg(test)]
mod test_utils {

    pub fn test_parse<T>(tokens: proc_macro2::TokenStream, expected: T)
    where
        T: syn::parse::Parse + PartialEq + std::fmt::Debug,
    {
        let actual = syn::parse2::<T>(tokens).expect("couldn't parse tokens");
        pretty_assertions::assert_eq!(expected, actual);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn trybuild() {
        let t = trybuild::TestCases::new();
        t.pass("trybuild/pass/**/*.rs");
        t.compile_fail("trybuild/fail/**/*.rs")
    }

    #[test]
    fn readme() {
        let expected = std::process::Command::new("cargo")
            .arg("readme")
            .output()
            .expect("couldn't run `cargo readme`")
            .stdout;
        let expected = String::from_utf8(expected).expect("`cargo readme` output wasn't UTF-8");
        let actual = include_str!("../README.md");
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_result_name() {
        let ident = get_struct_name_from_return_type(
            &syn::parse2(quote!(-> Result<T, SomeConcreteErr>)).unwrap(),
        )
        .unwrap();
        assert_eq!(ident, "SomeConcreteErr");

        let ident = get_struct_name_from_return_type(
            &syn::parse2(quote!(-> ::std::result::Result<T, SomeConcreteErr>)).unwrap(),
        )
        .unwrap();
        assert_eq!(ident, "SomeConcreteErr");
    }
}
