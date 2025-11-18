mod traced;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn traced(args: TokenStream, input: TokenStream) -> TokenStream {
    traced::traced(args, input)
}
