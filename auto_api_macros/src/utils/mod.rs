pub mod casing;

use auto_api_core::error::MacroError;

use std::fs;

/// Reads the strings of a resource given a URI, capable of reading resources
/// from the web and from your local filesystem
pub fn read_resource(uri: &str) -> Result<String, MacroError> {
    // Seperating the protocol and path from the uri
    let (protocol, path) = uri.split_once("://").ok_or(MacroError::InvalidInput(
        "Resource URI must contain '://' splitting protocol and path".to_string(),
    ))?;

    // Reading the contents based on the protocol
    let contents = match protocol {
        "http" | "https" => reqwest::blocking::get(uri)
            .and_then(|it| it.text())
            .map_err(|err| MacroError::ResourceLoadingFailed(err.to_string()))?,
        "file" => {
            // Getting the base directory (the one with Cargo.toml)
            let mut base_dir = String::default();
            if !path.starts_with("/") {
                base_dir.push_str(
                    &std::env::var("CARGO_MANIFEST_DIR")
                        .map_err(|err| MacroError::ResourceLoadingFailed(err.to_string()))?,
                );
            }

            fs::read_to_string(format!("{base_dir}/{path}"))
                .map_err(|err| MacroError::ResourceLoadingFailed(err.to_string()))?
        }
        _ => return Err(MacroError::UnsupportedProtocol(protocol.to_string())),
    };

    Ok(contents)
}
