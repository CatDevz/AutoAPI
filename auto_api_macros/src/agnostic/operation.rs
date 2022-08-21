use openapi::{v2, v3_0};
use proc_macro2::{Ident, Span};
use quote::quote;

use crate::utils::convert_casing_to_snake;

pub struct VersionAgnosticOperation {
    pub operation_id: Option<String>,
}

impl From<v2::Operation> for VersionAgnosticOperation {
    fn from(it: v2::Operation) -> Self {
        Self {
            operation_id: it.operation_id
        }
    }
}

impl From<v3_0::Operation> for VersionAgnosticOperation {
    fn from(it: v3_0::Operation) -> Self {
        Self {
            operation_id: it.operation_id
        }
    }
}

pub fn build_operation_method(path: &str, http_method: &str, operation: VersionAgnosticOperation) -> proc_macro2::TokenStream {
    // Getting the operation ID
    let operation_name = match operation.operation_id {
        Some(it) => it,
        None => {
            // If the API specification doesn't provide a operation id we build our own
            let mut endpoint = Vec::<&str>::new();
            endpoint.push(http_method);

            let split_path = path.split("/");
            for part in split_path {
                if part.contains("?") {
                    let (head, args) = part.split_once("?").unwrap();
                    endpoint.push(head);

                    let split_args = args.split("&");
                    for arg in split_args {
                        let arg = arg.split_once("=").unwrap().0;
                        endpoint.push(arg);
                    }
                } else {
                    endpoint.push(part);
                }
            }

            endpoint.join("_")
        },
    };
    
    // Getting the method name and identifier from the operation ID
    let method_name = convert_casing_to_snake(&operation_name);
    let method_ident = Ident::new(&method_name, Span::call_site());

    // Getting the method identifier
    let http_method_ident = Ident::new(&http_method.to_uppercase(), Span::call_site());

    quote! {
        pub async fn #method_ident (&self) -> Result<String, Error> {
            let res = self.inner.request( reqwest::Method::#http_method_ident, format!("{}{}", &self.base_url, #path) ).send().await.map_err(|_| Error::Unknown)?;
            let text = res.text().await.map_err(|_| Error::Unknown)?;
            Ok(text)
        }
    }
}
