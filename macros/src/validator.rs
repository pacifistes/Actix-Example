use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::DeriveInput;
use syn::Meta;
use syn::Token;

fn process_custom(
    field_name: &Option<&syn::Ident>,
    field_index: usize,
    is_option: bool,
    meta: &syn::MetaList,
    validators: &mut Vec<proc_macro2::TokenStream>,
) {
    let nested_arg = meta
        .parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
        .unwrap();
    for meta_arg in nested_arg {
        match meta_arg {
            Meta::NameValue(meta_name_value) if meta_arg.path().is_ident("function") => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = meta_name_value.value
                {
                    let function_path = lit_str.value();
                    // Parse the string into a path
                    let function_path = syn::parse_str::<syn::Path>(&function_path)
                        .expect("Failed to parse function path");

                    let params = quote! { identity };
                    let field = if let Some(field_name) = field_name {
                        quote! { self.#field_name }
                    } else {
                        let field_index = syn::Index::from(field_index);
                        quote! { self.#field_index }
                    };

                    if is_option {
                        validators.push(quote! {
                            if let Some(value) = &self.#field_name {
                                #function_path(#params, value).await?;
                            }
                        });
                    } else {
                        validators.push(quote! {
                            #function_path(#params, &#field).await?;
                        });
                    }
                }
            }
            _ => {
                println!("Unknown custom attribute: {:#?}", meta_arg);
            }
        }
    }
}

fn process_args(
    field_name: &Option<&syn::Ident>,
    field_index: usize,
    is_option: bool,
    nested_args: &syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>,
) -> Vec<proc_macro2::TokenStream> {
    let mut validators = Vec::new();
    for meta_arg in nested_args {
        match meta_arg {
            Meta::List(meta) if meta.path.is_ident("custom") => {
                process_custom(field_name, field_index, is_option, meta, &mut validators);
            }
            _ => {
                println!("Unknown validation attribute: {:#?}", meta_arg);
            }
        }
    }

    validators
}

fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Option")
            .unwrap_or(false)
    } else {
        false
    }
}

fn generate_validators(input: &DeriveInput) -> Vec<proc_macro2::TokenStream> {
    let mut validators = Vec::new();
    let mut field_index = 0;

    if let syn::Data::Struct(data) = &input.data {
        // field: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>
        for field in &data.fields {
            let field_name: Option<&syn::Ident> = field.ident.as_ref();
            for attr in &field.attrs {
                if attr.path().is_ident("custom_validate") {
                    let nested = attr
                        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .unwrap();

                    let is_option = is_option(&field.ty);
                    let validator = process_args(&field_name, field_index, is_option, &nested);
                    validators.extend(validator);
                }
            }
            field_index += 1;
        }
    }
    validators
}

pub fn validate_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("Input: {:#?}", input); // Debug output
    let name = &input.ident;

    let validators = generate_validators(&input);

    let expanded = quote! {
        impl CustomValidateTrait for #name {
            async fn validate(&self, identity: &Identity) -> Result<(), String> {
                #(#validators)*
                Ok(())
            }
        }
    };

    // println!("Generated code: {}", expanded); // Debug output
    TokenStream::from(expanded)
}
