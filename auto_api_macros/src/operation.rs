use auto_api_core::error::MacroError;
use openapiv3::{OpenAPI, Operation};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    documentation::generate_api_operation_docs,
    path::TypePathMap,
    utils::casing::{convert_casing_to_pascal, convert_casing_to_snake},
};

pub struct OperationMethod {
    pub builder_identifier: Ident,
    pub builder_declaration: TokenStream,
    pub function_identifier: Ident,
    pub function_declaration: TokenStream,
}

impl ToTokens for OperationMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.function_declaration.clone());
    }
}

pub fn generate_operation_method(
    type_map: &mut TypePathMap,
    path: &str,
    method: &str,
    operation: &Operation,
) -> Result<OperationMethod, MacroError> {
    // Generating docs with the operation
    let operation_docs = generate_api_operation_docs(&operation);

    // Getting the operation name, and making an identifier for the builder and function out of it
    let operation_name = match &operation.operation_id {
        Some(it) => it.clone(),
        None => format!("{method}/{path}")
            .replace("?", "/")
            .replace("&", "/")
            .split("/")
            .filter(|it| !it.is_empty())
            .collect::<Vec<&str>>()
            .join("_")
            .to_lowercase(),
    };

    let span = Span::call_site();
    let builder_identifier = Ident::new(&convert_casing_to_pascal(&operation_name), span);
    let function_identifier = Ident::new(&convert_casing_to_snake(&operation_name), span);

    // Building out the final AST for the function & builder declaration
    let mut builder_declaration = TokenStream::new();
    let mut function_declaration = TokenStream::new();

    if operation.deprecated {
        builder_declaration.extend(quote! {
            #[deprecated]
        });

        function_declaration.extend(quote! {
            #[deprecated]
        });
    }

    function_declaration.extend(quote! {
        #[doc = #operation_docs]
        pub async fn #function_identifier (&self) {
            // TODO:
        }
    });

    // Returning the final value
    Ok(OperationMethod {
        builder_identifier,
        builder_declaration,
        function_identifier,
        function_declaration,
    })
}

pub fn generate_operation_methods(
    type_map: &mut TypePathMap,
    api_spec: &OpenAPI,
) -> Result<Vec<OperationMethod>, MacroError> {
    let mut operations = Vec::<OperationMethod>::new();
    for (path, method, operation) in api_spec.operations() {
        operations.push(generate_operation_method(
            type_map, path, method, operation,
        )?);
    }

    Ok(operations)
}
