#![deny(missing_docs)]
//! Derive macros for `redact_composer`. Not needed as a direct dependency.

use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(element))]
struct Opts {
    name: Option<String>,
    wrapped_element: Option<Expr>,
    wrapped_element_doc: Option<String>,
}

/// Derives a `redact-composer` `Element` impl for this type.
///
/// The default implementation (which likely satisfies the majority of cases) is nothing more than:
/// ```ignore
/// # #[derive(Debug)]
/// # struct MyElement;
/// #[typetag::serde] // If "serde" feature enabled
/// impl Element for MyElement {}
/// ```
///
/// > *Important!: At the current time, if using the `serde` feature (enabled by default), in order
/// > to use this derive macro you need to have [`typetag`] added as a dependency to your
/// > crate.*
///
/// Additional options (if needed) are specified via the `#[element(params)]` attribute which accepts
/// any of the following params:
/// * `feature: serde`
///
///   **`name: String`:** Provides a different serialization name if you need to avoid naming collisions
///   or just prefer something different. In either case, this name is just passed along to
///   `#[typetag::serde(name = name)]`.
///
///   **Default:** the type's name.
///
/// * **`wrapped_element: Expr`:** If you are creating an Element that wraps
///   another you can specify the expression to access it (e.g. `Some(self.wrapped_item())`). The
///   expression should return an `Option<&dyn Element>`.
///
///   **Default:** `None`.
///
/// * **`wrapped_element_doc: String`:** Use this to provide a doc comment (no /// necessary) for the
///   wrapped element. Only has an effect if `wrapped_element` is also present.
#[proc_macro_derive(Element, attributes(element))]
pub fn derive(input: TokenStream) -> TokenStream {
    derive_impl(quote! { ::redact_composer }, input)
}

/// See [`Element`]. This version is used if only depending on `redact_composer_core` (i.e. for
/// lib development).
#[proc_macro_derive(ElementCore, attributes(element))]
pub fn core_derive(input: TokenStream) -> TokenStream {
    derive_impl(quote! { ::redact_composer_core }, input)
}

fn derive_impl(crate_path: proc_macro2::TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Invalid element option");
    let DeriveInput { ident, .. } = input;

    let wrapped_element_comment = if let Some(comment) = opts.wrapped_element_doc {
        quote! { #[doc= #comment ] }
    } else {
        quote! { #[doc= "Wrapped element." ] }
    };

    let wrapped_element_accessor = match opts.wrapped_element {
        Some(accessor) => quote! {
            #wrapped_element_comment
            fn wrapped_element(&self) -> Option<&dyn Element> {
                #accessor
            }
        },
        None => quote! {},
    };

    let typetag_attr = if cfg!(feature = "serde") {
        let type_tag_opts = match opts.name {
            Some(name_opt) => quote! { (name = #name_opt) },
            None => quote! {},
        };

        quote! { #[typetag::serde #type_tag_opts] }
    } else {
        quote! {}
    };

    let output = quote! {
        #typetag_attr
        impl #crate_path::Element for #ident {
            #wrapped_element_accessor
        }
    };

    output.into()
}
