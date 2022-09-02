use auto_api_core::error::MacroError;
use serde::de::DeserializeOwned;

/// Expand the reference from a root object (node), as an example '#/contacts/email'
/// will point to the email value in the contacts object in the root object
///
/// Note: Does not support references to external resources ('other.yaml#/contacts')
pub fn expand_reference<'a>(
    root_node: &'a serde_json::Value,
    reference: &str,
) -> Result<&'a serde_json::Value, MacroError> {
    if !reference.starts_with("#") {
        return Err(MacroError::UnimplementedFeature(
            "non-local references are unsupported".to_string(),
        ));
    }

    // Traversing the root until we end up with the node we wanted
    let mut parts = reference.split("/").skip(1);
    let mut current_node = root_node;
    while let Some(next) = parts.next() {
        current_node = current_node
            .get(next)
            .ok_or_else(|| MacroError::InvalidReference(reference.to_string()))?;
    }

    Ok(current_node)
}

/// Expand the reference from root object and then deserialize into type T.
pub fn expand_typed_reference<T: DeserializeOwned>(
    root_node: &serde_json::Value,
    reference: &str,
) -> Result<T, MacroError> {
    let expanded = expand_reference(root_node, reference)?;
    let expanded = serde_json::from_value(expanded.clone())
        .map_err(|err| MacroError::InvalidInput(err.to_string()))?;
    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::expand_reference;

    #[test]
    fn test_expand_reference() {
        let input = json!({
            "day": 2,
            "contacts": {
                "email": "testing@example.com",
                "phone": "+1 800-729-4625",
                "address": {
                    "country": "United States",
                    "zip": 58205,
                    "street": "6294 Bodega Street"
                }
            }
        });

        let res = expand_reference(&input, "#/day").unwrap();
        assert_eq!(res, &serde_json::Value::Number(2.into()));

        let res = expand_reference(&input, "#/contacts/address/zip").unwrap();
        assert_eq!(res, &serde_json::Value::Number(58205.into()));

        let res = expand_reference(&input, "#/contacts/phone").unwrap();
        assert_eq!(res, &serde_json::Value::String("+1 800-729-4625".into()));
    }
}
