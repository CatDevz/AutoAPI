use std::collections::HashMap;

use auto_api_core::error::MacroError;
use proc_macro2::Span;
use quote::quote;
use serde::{Deserialize, Serialize};

use crate::{
    reference::{expand_reference, expand_typed_reference},
    utils::casing::{convert_casing_to_pascal, convert_casing_to_snake},
};

// - Go through creating endpoints
// - When we find a model we check if structs & enums have been generated for it
//   - If that is the case we generate the structs & enums required for it (recursive)
// - We can reference the model

// Input
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

// Output
#[derive(Clone)]
pub enum GenerationResult {
    Property(SchemaProperty),
    Model(SchemaModel),
}

#[derive(Clone)]
pub struct SchemaProperty {
    pub identifier: proc_macro2::Ident,
    pub definition: proc_macro2::TokenStream,
}

#[derive(Clone)]
pub struct SchemaModel {
    pub identifier: proc_macro2::Ident,
    pub definition: proc_macro2::TokenStream,
}

/// Recursively generate models out of schemas
///
/// Schema       - OpenAPI concept of a Schema
/// Schema Json  - Rust JSON Value representation of Schema
/// Schema Model - Rust model defining Schema
/// Schema Impl  - Rust code implementing Schema
fn pog(
    root: &serde_json::Value,
    models: &mut HashMap<String, SchemaModel>,
    schema: RefOrObject<Schema>,
    name: &str,
) -> Result<(), MacroError> {
    // If the schema is a reference we can expand the reference
    let schema = match schema {
        RefOrObject::Object(it) => it,
        RefOrObject::Ref { ref_path } => {
            // If the schema already has a model implemented for it there is no need to rengenerate
            if let Some(existing_model) = models.get(&ref_path) {
                return Ok(());
            }

            // Expanding the schema and determining a name
            let expanded = expand_typed_reference::<RefOrObject<Schema>>(&root, &ref_path)?;
            let name = ref_path
                .split("/")
                .last()
                .ok_or(MacroError::InvalidInput("Invalid path".to_string()))?;

            return pog(root, models, expanded, name);
        }
    };

    // Generating the identifier and name for our schema
    let name = match schema {
        Schema::Basic(BasicSchema::String {}) => {}
        Schema::Basic(BasicSchema::Number {}) => {}
        Schema::Basic(BasicSchema::Integer {}) => {}
        Schema::Basic(BasicSchema::Boolean {}) => {}
        Schema::Basic(BasicSchema::Array { items: _ }) => {}
        Schema::Basic(BasicSchema::Object { properties: _ }) => {}
        Schema::Union(UnionSchema::OneOf { one_of: _ }) => {}
        Schema::Union(UnionSchema::AnyOf { any_of: _ }) => {}
        Schema::Union(UnionSchema::AllOf { all_of: _ }) => {}
        Schema::Union(UnionSchema::Not { not: _ }) => {
            // FIXME: Add support for 'not'
            return Err(MacroError::UnimplementedFeature("'not' schema".to_string()));
        }
    };

    todo!()
}

pub fn gen_schema_impl(
    existing_models: &mut HashMap<String, SchemaModel>,
    root: &serde_json::Value,
    schema_path: &str,
    schema_base_name: &str,
) -> Result<GenerationResult, MacroError> {
    // Getting the schema from the root node and the schema path
    let schema_json = expand_reference(&root, schema_path)?;
    let schema_model = serde_json::from_value::<RefOrObject<Schema>>(schema_json.clone())
        .map_err(|err| MacroError::InvalidInput(err.to_string()))?;

    // If the model is a reference we go deeper!
    let schema_model = match schema_model {
        RefOrObject::Object(it) => it,
        RefOrObject::Ref { ref_path } => {
            return match existing_models.get(&ref_path) {
                // If the schema implementation has already been generated we can just return without going deeper
                // FIXME: Get rid of this clone
                Some(existing_model) => Ok(GenerationResult::Model(existing_model.clone())),
                None => gen_schema_impl(existing_models, root, &ref_path, schema_base_name),
            };
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
    let identifier = proc_macro2::Ident::new(&name, Span::call_site());

    // TODO: Getting any references and making sure they have been generated, otherwise making a call to generate them
    // TODO: Generating the code for our schema
    let mut schema_impl_code = proc_macro2::TokenStream::new();

    match schema_model {
        Schema::Basic(BasicSchema::String {}) => {
            let name = convert_casing_to_snake(&format!("{schema_base_name}{name}"));
            let identifier = proc_macro2::Ident::new(&name, Span::call_site());
            let definition = quote!(#identifier: String);

            return Ok(GenerationResult::Property(SchemaProperty {
                identifier,
                definition,
            }));
        }
        Schema::Basic(BasicSchema::Number {}) | Schema::Basic(BasicSchema::Integer {}) => {
            let name = convert_casing_to_snake(&format!("{schema_base_name}{name}"));
            let identifier = proc_macro2::Ident::new(&name, Span::call_site());
            let definition = quote!(#identifier: i32);

            return Ok(GenerationResult::Property(SchemaProperty {
                identifier,
                definition,
            }));
        }
        Schema::Basic(BasicSchema::Boolean {}) => {
            let name = convert_casing_to_snake(&format!("{schema_base_name}{name}"));
            let identifier = proc_macro2::Ident::new(&name, Span::call_site());
            let definition = quote!(#identifier: bool);

            return Ok(GenerationResult::Property(SchemaProperty {
                identifier,
                definition,
            }));
        }
        Schema::Basic(BasicSchema::Array { items: _ }) => {
            todo!();
        }
        Schema::Basic(BasicSchema::Object { properties }) => {
            // TODO: Generate struct
            for (key, inner) in properties.iter() {}

            ""
        }
        Schema::Union(UnionSchema::OneOf { one_of: _ }) => {
            // TODO: Generate enum

            "todo!()"
        }
        Schema::Union(UnionSchema::AnyOf { any_of: _ }) => "todo!()",
        Schema::Union(UnionSchema::AllOf { all_of: _ }) => "todo!()",
        Schema::Union(UnionSchema::Not { not: _ }) => {
            // FIXME: Add support for 'not'
            return Err(MacroError::UnimplementedFeature("'not' schema".to_string()));
        }
    };

    let schema_impl_code = quote! {
        pub struct #identifier {
            #schema_impl_code
        }
    };

    // Making a resulting schema model and adding it to our generated_schemas hashmap
    let schema_impl = SchemaModel {
        identifier: proc_macro2::Ident::new("string", Span::call_site()),
        definition: schema_impl_code,
    };

    // FIXME: Remove this clone aswell
    existing_models.insert(schema_path.to_string(), schema_impl.clone());

    Ok(GenerationResult::Model(schema_impl))
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
