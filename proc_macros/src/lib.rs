//! Procedural macros for the Navign project
//!
//! This crate provides custom derive macros and attribute macros
//! for use across the Navign indoor navigation system.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Example derive macro - replace with actual implementation
///
/// # Example
///
/// ```ignore
/// use navign_proc_macros::ExampleDerive;
///
/// #[derive(ExampleDerive)]
/// struct MyStruct {
///     field: String,
/// }
/// ```
#[proc_macro_derive(ExampleDerive)]
pub fn example_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn example_method(&self) {
                println!("Example method called on {}", stringify!(#name));
            }
        }
    };

    TokenStream::from(expanded)
}

/// Example attribute macro - replace with actual implementation
///
/// # Example
///
/// ```ignore
/// use navign_proc_macros::example_attribute;
///
/// #[example_attribute]
/// fn my_function() {
///     // function body
/// }
/// ```
#[proc_macro_attribute]
pub fn example_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass through unchanged for now
    item
}

/// Example function-like macro - replace with actual implementation
///
/// # Example
///
/// ```ignore
/// use navign_proc_macros::example_macro;
///
/// example_macro! {
///     // macro input
/// }
/// ```
#[proc_macro]
pub fn example_macro(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        // Generated code
    };

    TokenStream::from(expanded)
}
