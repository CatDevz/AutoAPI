pub(crate) mod agnostic;
pub(crate) mod args;
pub(crate) mod model;
pub(crate) mod reference;
pub(crate) mod utils;

use agnostic::operation::{build_operation_method, VersionAgnosticOperation};
use args::GenApiArguments;
use utils::read_resource;

use openapi::v2::Scheme;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;

/// Generate a client library from the provided OpenAPI/Swagger specification. This specification
/// can be provided either from online, or from a local file (recommended).
///
/// Example usage:
///
/// ```ignore
/// #[auto_api::gen_api("file:///openapi/petstore.json")]
/// pub mod petstore_api_local {}
/// ```
///
/// TODO: Write better documentation with more complete examples
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
                note = "Auto API currently only supports OpenAPI version 3.0.0";
                help = "Try opening {} in a web browser to verify it's resp&onding with the correct documentation",& &openapi_uri;
        ));

    // Getting a description from the OpenAPI documentation to use as docs on our Client struct
    let openapi_description = match &openapi_spec {
        openapi::OpenApi::V2(v2) => {
            let info = &v2.info;
            let mut res = String::default();

            if let Some(title) = &info.title {
                res.push_str("# ");
                res.push_str(title);
                res.push_str("\n\n");
            }

            if let Some(description) = &info.description {
                res.push_str(description);
                res.push_str("\n\n");
            }

            if let Some(tos) = &info.terms_of_service {
                res.push_str("[Terms of Service](");
                res.push_str(tos);
                res.push_str(")");
            }

            res
        }
        openapi::OpenApi::V3_0(v3) => {
            let info = &v3.info;
            let mut res = String::default();

            res.push_str("# ");
            res.push_str(&info.title);
            res.push_str("\n\n");

            if let Some(description) = &info.description {
                res.push_str(description);
                res.push_str("\n\n");
            }

            if let Some(tos) = &info.terms_of_service {
                res.push_str("[Terms of Service](");
                res.push_str(&tos.to_string());
                res.push_str(")");
            }

            res
        }
    };

    // Getting the base URL of the API
    // TODO: Change to use a server enum
    let base_url = match openapi_spec.clone() {
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
                    arguments.documentation, "Scheme detected is not supported.";
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
        abort!(arguments.documentation,
            "Could not determine base url from OpenAPI documentation.";
                help = "You can supply a base url as an optional 3rd parameter";
        )
    });

    let _servers = match openapi_spec.clone() {
        openapi::OpenApi::V2(_v2) => {}
        openapi::OpenApi::V3_0(v3) => {
            let servers = v3.servers.unwrap_or_default();
            for (i, server) in servers.iter().enumerate() {
                let _name = server.description.clone().unwrap_or_else(|| i.to_string());
            }
            // servers.iter().enumerate().map(|(i, server)| {
            //     let default_name = i.to_string();
            //     let name = server
            //         .description
            //         .as_ref()
            //         .and_then(|it| it.split(" ").next())
            //         .unwrap_or_else(|| &default_name);
            // });
        }
    };

    // Generating our API methods
    let mut _models = Vec::<proc_macro2::TokenStream>::new();
    let mut methods = Vec::<proc_macro2::TokenStream>::new();

    match openapi_spec.clone() {
        openapi::OpenApi::V2(v2) => {
            for (path, details) in v2.paths.iter() {
                for (http_method, operation) in details.iter() {
                    methods.push(build_operation_method(
                        path,
                        &http_method,
                        VersionAgnosticOperation::from(operation.clone()),
                    ));
                }
            }
        }
        openapi::OpenApi::V3_0(v3) => {
            for (path, details) in v3.paths.iter() {
                if let Some(operation) = &details.get {
                    methods.push(build_operation_method(
                        path,
                        "GET",
                        VersionAgnosticOperation::from(operation.clone()),
                    ));
                }
                // TODO: More than just GET
            }
        }
    };

    // Generating the output token stream
    let inner = quote! {
        #![doc = #openapi_description]

        // Include some private runtime dependencies.
        use auto_api::__private::{reqwest, serde};

        use auto_api::client::base::{BaseClient, BaseServer, ClientOptions};
        use auto_api::error::Error;

        // Server
        pub enum Server {
            Custom { url: String },
        }

        impl BaseServer for Server {
            fn url(&self) -> &str {
                match self {
                    Server::Custom { url } => url,
                }
            }
        }

        impl Default for Server {
            fn default() -> Self {
                // TODO:
                Self::Custom {
                    url: #base_url.to_string()
                }
            }
        }

        // Client
        #[doc = #openapi_description]
        pub struct Client {
            inner: reqwest::Client,
            pub base_url: String,
        }

        impl Client {
            #( #methods )*
        }

        impl BaseClient<Server> for Client {
            fn new(options: ClientOptions<Server>) -> Self {
                Self {
                    inner: reqwest::Client::new(),
                    base_url: options.server.url().to_string(),
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
