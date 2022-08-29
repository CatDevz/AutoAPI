use std::collections::HashMap;

use auto_api_core::error::MacroError;
use proc_macro2::Span;
use quote::quote;
use serde::{Deserialize, Serialize};

use crate::{reference::expand_reference, utils::casing::convert_casing_to_pascal};

// - Go through creating endpoints
// - When we find a model we check if structs & enums have been generated for it
//   - If that is the case we generate the structs & enums required for it (recursive)
// - We can reference the model

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum RefOrObject<T> {
    // First check if it's a reference to something somewhere else or is T inlined
    Object(T),
    Ref {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Schema {
    Basic(BasicSchema),
    Union(UnionSchema),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum BasicSchema {
    // Then check if anything matches Schema, if not then move on to validating UnionSchema
    #[serde(rename = "string")]
    String {},
    #[serde(rename = "number")]
    Number {},
    #[serde(rename = "integer")]
    Integer {},
    #[serde(rename = "boolean")]
    Boolean {},
    #[serde(rename = "array")]
    Array { items: Box<RefOrObject<Schema>> },
    #[serde(rename = "object")]
    Object {
        properties: Box<HashMap<String, RefOrObject<Schema>>>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum UnionSchema {
    // Finally check if the schema is one of the union schema types.
    OneOf {
        #[serde(rename = "oneOf")]
        one_of: Box<Vec<RefOrObject<Schema>>>,
    },
    AnyOf {
        #[serde(rename = "anyOf")]
        any_of: Box<Vec<RefOrObject<Schema>>>,
    },
    AllOf {
        #[serde(rename = "allOf")]
        all_of: Box<Vec<RefOrObject<Schema>>>,
    },
    Not {
        not: Box<RefOrObject<Schema>>,
    },
}

pub struct SchemaImpl {
    pub identifier: proc_macro2::Ident,
    pub tokens: proc_macro2::TokenStream,
}

/// Recursively generate schema models
///
/// Schema       - OpenAPI concept of a Schema
/// Schema Json  - Rust JSON Value representation of Schema
/// Schema Model - Rust model defining Schema
/// Schema Impl  - Rust code implementing Schema
pub fn gen_schema_impl(
    mut schema_impls: HashMap<String, SchemaImpl>,
    root: &serde_json::Value,
    schema_path: &str,
    schema_base_name: &str,
) -> Result<HashMap<String, SchemaImpl>, MacroError> {
    // Getting the schema from the root node and the schema path
    let schema_json = expand_reference(&root, schema_path)?;
    let schema_model = serde_json::from_value::<RefOrObject<Schema>>(schema_json.clone())
        .map_err(|err| MacroError::InvalidInput(err.to_string()))?;

    // If the model is a reference we go deeper!
    let schema_model = match schema_model {
        RefOrObject::Object(it) => it,
        RefOrObject::Ref { ref_path } => {
            // If the schema implementation has already been generated we can just return early
            if schema_impls.contains_key(&ref_path) {
                return Ok(schema_impls);
            }

            return gen_schema_impl(schema_impls, root, &ref_path, schema_base_name);
        }
    };

    // Determining a name for our schema
    let name = schema_path
        .split("/")
        .last()
        .ok_or(MacroError::InvalidInput("Invalid path".to_string()))?;

    let name = schema_json
        .get("xml")
        .and_then(|it| it.as_object())
        .and_then(|it| it.get("name"))
        .and_then(|it| it.as_str())
        .unwrap_or(name)
        .to_string();

    let name = convert_casing_to_pascal(&format!("{schema_base_name}{name}"));

    // FIXME: Add support for 'not'
    if let Schema::Union(UnionSchema::Not { not: _ }) = schema_model {
        return Err(MacroError::UnimplementedFeature("'not' schema".to_string()));
    }

    // TODO: Getting any references and making sure they have been generated, otherwise making a call to generate them
    // TODO: Generating the code for our schema
    let schema_impl_identifier = proc_macro2::Ident::new(&name, Span::call_site());
    let mut schema_impl = proc_macro2::TokenStream::new();

    // Making a resulting schema model and adding it to our generated_schemas hashmap
    let schema_impl = SchemaImpl {
        identifier: proc_macro2::Ident::new("string", Span::call_site()),
        tokens: quote! {},
    };

    schema_impls.insert(schema_path.to_string(), schema_impl);

    Ok(schema_impls)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{RefOrObject, Schema};

    #[test]
    fn deserializing_basic_schema() {
        let input = json!({
            "type": "array",
            "items": {
                "oneOf": [
                    {
                        "$ref": "#/hi/hello/hola"
                    },
                    {
                        "type": "string"
                    },
                    {
                        "type": "object",
                        "properties": {
                            "welcomeMessage": {
                                "type": "string"
                            },
                            "welcomeType": {
                                "$ref": "#/hi/hello/hola"
                            }
                        }
                    }
                ]
            }
        });

        let output = serde_json::from_value::<RefOrObject<Schema>>(input).unwrap();

        println!("{:#?}", output);

        assert_eq!(2 + 2, 5);
    }
}
