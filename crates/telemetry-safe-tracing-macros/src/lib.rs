//! Proc macros for `telemetry-safe-tracing`.
//!
//! `safe_instrument` will live here so the public tracing crate can stay a
//! normal library and still expose helper types alongside the attribute macro.

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn safe_instrument(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);

    // A passthrough implementation would silently reintroduce the exact
    // footgun this crate exists to remove, so the unfinished macro fails closed.
    quote! {
        compile_error!("safe_instrument is not implemented yet");
        #item
    }
    .into()
}
