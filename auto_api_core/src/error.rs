#[derive(thiserror::Error, Debug, PartialEq)]
pub enum MacroError {
    #[error("Invalid input was provided. {0}")]
    InvalidInput(String),
    #[error("The reference ($ref) '{0}' is invalid.")]
    InvalidReference(String),
    #[error("The protocol '{0}' is unsupported")]
    UnsupportedProtocol(String),
    #[error("Failed to load resource. {0}")]
    ResourceLoadingFailed(String),
    #[error("The {0} feature of the OpenAPI spec is currently unimplemented in AutoAPI")]
    UnimplementedFeature(String),
    #[error("An unknown error ocurred")]
    Unknown,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("An unknown error ocurred")]
    Unknown,
}
