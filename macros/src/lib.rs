use proc_macro::TokenStream;

mod validator;

#[proc_macro_derive(CustomValidate, attributes(custom_validate))]
pub fn validate_derive(input: TokenStream) -> TokenStream {
    validator::validate_derive(input)
}
