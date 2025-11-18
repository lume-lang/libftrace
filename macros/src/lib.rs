#[cfg(feature = "enabled")]
mod traced;

use proc_macro::TokenStream;

#[cfg(feature = "enabled")]
#[proc_macro_attribute]
pub fn traced(args: TokenStream, input: TokenStream) -> TokenStream {
    traced::traced(args, input)
}

#[cfg(not(feature = "enabled"))]
#[proc_macro_attribute]
pub fn traced(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
