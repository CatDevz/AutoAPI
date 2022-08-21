pub(crate) mod utils;
pub(crate) mod agnostic;

use agnostic::operation::{build_operation_method, VersionAgnosticOperation};
use utils::read_resource;

use openapi::v2::Scheme;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::parse::{Parse, ParseStream};

struct GenApiArguments {
    documentation: syn::LitStr,
    api_base_url: Option<syn::LitStr>,
}

impl Parse for GenApiArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // jank™.
        struct GenApiSyntax(syn::LitStr, Option<syn::token::Comma>, Option<syn::LitStr>);
        let syntax = GenApiSyntax(
            input.parse()?,
            input.parse().unwrap_or(None),
            input.parse().unwrap_or(None),
        );

        Ok(GenApiArguments {
            documentation: syntax.0,
            api_base_url: syntax.2,
        })
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn gen_api(arguments: TokenStream, module: TokenStream) -> TokenStream {
    let arguments = syn::parse_macro_input!(arguments as GenApiArguments);

    // Getting the head & tail of the item
    let module_str = module.to_string();
    let (module_head, module_tail) = module_str.split_once("{").unwrap();

    // Validating the attribute is on a module
    if !module_head.contains("mod") {
        abort!(
            arguments.documentation,
            "This attribute must be applied on a module"
        );
    }

    // Downloading the OpenAPI documentation from the URL provided
    let openapi_uri = arguments.documentation.value();
    let openapi_text = read_resource(&openapi_uri)
        .unwrap_or_else(|err| abort!(
            arguments.documentation, "{}", &err;
                note = "If you are using a remote resource an internet connection is required to compile";
        ));

    // Parsing the OpenAPI documentation
    let openapi_spec = openapi::from_reader(openapi_text.as_bytes())
        .unwrap_or_else(|_| abort!(
            arguments.documentation, "OpenAPI documentation provided is of an unsupported format/version or malformed."; 
                note = "Auto API currently only supports OpenAPI versions 2.0 and 3.0.0";
                help = "Try opening {} in a web browser to verify it's resp&onding with the correct documentation",& &openapi_uri;
        ));

    // Getting the base URL of the API
    let base_url = match arguments.api_base_url {
        Some(it) => it.value(),
        None => {
            // If no specific base URL was provided we try to do the best we can with the OpenAPI documentation provided.
            match openapi_spec.clone() {
                openapi::OpenApi::V2(v2) => {
                    // Getting the scheme or defaulting to https
                    let scheme = match v2
                        .schemes
                        .unwrap_or_default()
                        .first()
                        .unwrap_or(&Scheme::Https)
                    {
                        Scheme::Http => "http",
                        Scheme::Https => "https",
                        Scheme::Ws | Scheme::Wss => abort!(
                            arguments.api_base_url, "Scheme detected is not supported.";
                                help = "You can supply a base url as an optional 3rd parameter";
                        ),
                    };

                    let base_path = v2.base_path.unwrap_or_default();
                    v2.host
                        .map(|it| format!("{}://{}{}", scheme, it, base_path))
                }
                openapi::OpenApi::V3_0(v3) => v3
                    .servers
                    .and_then(|it| it.first().cloned())
                    .map(|it| it.url),
            }
            .unwrap_or_else(|| {
                abort!(arguments.api_base_url,
                    "Could not determine base url from OpenAPI documentation.";
                        help = "You can supply a base url as an optional 3rd parameter";
                )
            })
        }
    };

    // Generating our API methods
    let mut _models = Vec::<proc_macro2::TokenStream>::new();
    let mut methods = Vec::<proc_macro2::TokenStream>::new();

    match openapi_spec.clone() {
        openapi::OpenApi::V2(v2) => {
            for (path, details) in v2.paths.iter() {
                for (http_method, operation) in details.iter() {
                    methods.push(build_operation_method(path, &http_method, VersionAgnosticOperation::from(operation.clone())));
                }
            }
        }
        openapi::OpenApi::V3_0(v3) => {
            for (path, details) in v3.paths.iter() {
                if let Some(operation) = &details.get {
                    methods.push(build_operation_method(path, "GET", VersionAgnosticOperation::from(operation.clone())));
                }
            }
        }
    };

    // Generating the output token stream
    let inner = quote! {
        use auto_api::reqwest;
        use auto_api::client::base::{BaseClient, ClientOptions};
        use auto_api::error::Error;

        use serde::{Serialize, Deserialize};

        pub struct Client {
            inner: reqwest::Client,
            pub base_url: String,
        }

        impl Client {
            #( #methods )*
        }

        impl BaseClient for Client {
            fn default_base_url() -> &'static str {
                #base_url
            }

            fn new(options: ClientOptions<Client>) -> Self {
                Self {
                    inner: reqwest::Client::new(),
                    base_url: options.base_url.to_string(),
                }
            }
        }

        impl Default for Client {
            fn default() -> Self {
                Self::new(Default::default())
            }
        }
    };

    // Building the resulting AST
    // Once again, jank™ but this was the easiest way I could find.
    format!("{}{{{}{}", module_head, inner.to_string(), module_tail)
        .parse()
        .unwrap()
}
