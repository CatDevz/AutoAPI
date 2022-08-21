#[derive(thiserror::Error, Debug)]
pub enum MacroError {
    #[error("Invalid input was provided. {details}")]
    InvalidInput { details: String },
    #[error("The protocol '{protocol}' is unsupported")]
    UnsupportedProtocol { protocol: String },
    #[error("Failed to load resource. {details}")]
    ResourceLoadingFailed { details: String },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An unknown error ocurred")]
    Unknown
}
