pub(crate) mod args;
pub(crate) mod documentation;
pub(crate) mod operation;
pub(crate) mod path;
pub(crate) mod utils;

use args::{GenApiArguments, GenApiModule};
use documentation::generate_api_module_docs;
use openapiv3::OpenAPI;
use operation::generate_operation_methods;
use path::TypePathMap;
use utils::read_resource;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;

/// Generate a client library from the provided OpenAPI/Swagger specification. This specification
/// can be provided either from online, or from a local file (recommended).
///
/// Example usage:
///
/// ```ignore
/// #[auto_api::gen_api("file://openapi/petstore.json")]
/// pub mod petstore_api_local {}
/// ```
///
/// TODO: Write better documentation with more complete examples
#[proc_macro_attribute]
#[proc_macro_error]
pub fn gen_api(arguments: TokenStream, module: TokenStream) -> TokenStream {
    let arguments = syn::parse_macro_input!(arguments as GenApiArguments);
    let module = syn::parse_macro_input!(module as GenApiModule);

    // Downloading the OpenAPI documentation from the URL provided
    let openapi_uri = arguments.documentation.value();
    let openapi_text = read_resource(&openapi_uri)
        .unwrap_or_else(|err| abort! {
            arguments.documentation, "{}", &err;
                note = "If you are using a remote resource an internet connection is required to compile";
        });

    // Parsing the OpenAPI documentation
    let api_spec = serde_json::from_str::<OpenAPI>(&openapi_text)
        .unwrap_or_else(|_| abort! {
            arguments.documentation, "OpenAPI documentation provided is of an unsupported format/version or malformed.";
                note = "Auto API currently only supports OpenAPI version 3.0.0";
                help = "Try opening {} in a web browser to verify it's resp&onding with the correct documentation",& &openapi_uri;
        });

    // Getting some meta-data about the API to generate documentation with
    let module_documentation = generate_api_module_docs(&api_spec.info);

    // Creating a type map and filling it with types in components/schema
    let mut type_map = TypePathMap::new();

    // Getting a list of generated operation methods
    let operation_methods = generate_operation_methods(&mut type_map, &api_spec).unwrap();

    // Building the resulting AST
    let module_visibility = module.module_visibility;
    let module_ident = module.module_ident;
    let module_content = module.module_content;

    quote! {
        #module_visibility mod #module_ident {
            #![doc = #module_documentation]

            // Include some private runtime dependencies.
            use auto_api::__private::{reqwest, serde};

            // Making the client
            #[doc = #module_documentation]
            pub struct Client {
                inner: reqwest::Client,
            }

            impl Client {
                #( #operation_methods )*
            }

            #module_content
        }
    }
    .into()
}
