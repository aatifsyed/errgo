use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use syn::{parse2, parse_macro_input, visit_mut::VisitMut, ItemFn};

mod data;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn err_as_you_go(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    pretty_env_logger::try_init_custom_env("RUST_LOG_ERR_AS_YOU_GO").ok();

    //////////////////////
    // Parse our inputs //
    //////////////////////
    let attr = TokenStream::from(attr);
    if !attr.is_empty() {
        emit_error!(attr, "arguments to this attribute are not supported")
    }
    let mut item = parse_macro_input!(item as ItemFn);

    let error_name = Ident::new(
        &format!(
            "{}Error",
            heck::AsUpperCamelCase(item.sig.ident.to_string())
        ),
        Span::call_site(),
    );
    let error_vis = item.vis.clone();

    let mut collector = CollectErrorVariants::new(error_name.clone());
    collector.visit_item_fn_mut(&mut item);

    for (src, e) in collector.collection_errors {
        emit_error!(src, "{}", e)
    }

    let variants = collector.variants.iter().map(|it| &it.ident);

    quote! {
        #error_vis enum #error_name {
            #(#variants),*
        }

        #item
    }
    .into()
}

/// Implementation detail
#[doc(hidden)]
#[proc_macro]
#[proc_macro_error]
pub fn __nothing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}

struct ErrorVariant {
    ident: Ident,
}

struct CollectErrorVariants {
    error_name: Ident,
    variants: Vec<ErrorVariant>,
    collection_errors: Vec<(TokenStream, syn::Error)>,
}

impl CollectErrorVariants {
    fn new(error_name: Ident) -> Self {
        Self {
            error_name,
            variants: Vec::new(),
            collection_errors: Vec::new(),
        }
    }
}

impl syn::visit_mut::VisitMut for CollectErrorVariants {
    fn visit_macro_mut(&mut self, i: &mut syn::Macro) {
        if i.path.is_ident("err") {
            match parse2::<Ident>(i.tokens.clone()) {
                Ok(ident) => {
                    self.variants.push(ErrorVariant {
                        ident: ident.clone(),
                    });
                    i.path = path(["err_as_you_go", "__nothing"]);
                    let error_name = &self.error_name;
                    i.tokens = quote!(#error_name::#ident);
                }
                Err(e) => self.collection_errors.push((i.tokens.clone(), e)),
            }
        }
    }
}

fn path<'a>(segments: impl IntoIterator<Item = &'a str>) -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: syn::punctuated::Punctuated::from_iter(
            segments
                .into_iter()
                .map(|it| syn::PathSegment::from(syn::Ident::new(it, Span::call_site()))),
        ),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let t = trybuild::TestCases::new();
        t.pass("trybuild/pass/**/*.rs");
        t.compile_fail("trybuild/fail/**/*.rs")
    }
}
