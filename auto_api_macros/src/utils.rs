use auto_api_core::error::MacroError;

use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::fs;

/// Converts the casing of the inputed value from camelCase, PascalCase or 
/// SCREAMING_SNAKE_CASE to the standard snake_case.
pub fn convert_casing_to_snake(original: &str) -> String {
    lazy_static! {
        // This regex is used to split a camelCase, PascalCase or SCREAMING_SNAKE_CASE 
        // string. Usage: https://regex101.com/r/mJW2yk/1
        static ref CASING_SPLIT_REGEX: Regex = {
            Regex::new(r"(?>[A-Z]?)[a-z0-9]+|[A-Z]+").unwrap()
        };
    }

    let result = CASING_SPLIT_REGEX
        .captures_iter(&original)
        .filter_map(|it| it.ok())
        .filter_map(|it| it.get(0))
        .map(|it| it.as_str())
        .collect::<Vec<&str>>();
    result.join("_").to_lowercase()
}

/// Reads the strings of a resource given a URI, capable of reading resources
/// from the web and from your local filesystem
pub fn read_resource(uri: &str) -> Result<String, MacroError> {
    // Seperating the protocol and path from the uri
    let (protocol, path) = uri.split_once("://")
        .ok_or(MacroError::InvalidInput { details: "Resource URI must contain '://' splitting protocol and path".to_string() })?;

    // Reading the contents based on the protocol
    let contents = match protocol {
        "http" | "https" => {
            reqwest::blocking::get(uri)
                .and_then(|it| it.text())
                .map_err(|err| MacroError::ResourceLoadingFailed { details: err.to_string() })?
        }
        "file" => {
            fs::read_to_string(&path)
                .map_err(|err| MacroError::ResourceLoadingFailed { details: err.to_string() })?
        }
        _ => return Err(MacroError::UnsupportedProtocol { protocol: protocol.to_string() }),
    };

    Ok(contents)
}
