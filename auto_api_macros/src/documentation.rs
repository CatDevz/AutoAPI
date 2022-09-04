use openapiv3::{Info, Operation};

pub fn generate_api_module_docs(api_info: &Info) -> String {
    let mut result = String::new();

    // Adding on the title & version
    {
        result.push_str("# ");
        result.push_str(&api_info.title);
        result.push_str(" (Version ");
        result.push_str(&api_info.version);
        result.push_str(")");
        result.push_str("\n\n");
    }

    // Adding on the description, if available
    if let Some(description) = &api_info.description {
        result.push_str(description);
        result.push_str("\n\n");
    }

    // Adding external docs, if available
    if let Some(terms_of_service) = &api_info.terms_of_service {
        result.push_str("**[Terms of Service](");
        result.push_str(terms_of_service);
        result.push_str(")**");
    }

    result
}

pub fn generate_api_operation_docs(operation: &Operation) -> String {
    let mut result = String::new();

    // Adding on the summary
    if let Some(summary) = &operation.summary {
        result.push_str(summary);
        result.push_str("\n\n");
    }

    // Adding on the description
    if let Some(description) = &operation.description {
        result.push_str(description);
        result.push_str("\n\n");
    }

    // Adding info if the function is deprecated
    if operation.deprecated {
        result.push_str("**This operation is deprecated**");
    }

    result
}
