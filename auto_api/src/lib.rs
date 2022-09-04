//! This crate provides the `gen_api` macro for generating a client library
//! based on a provided OpenAPI or Swagger compliant document.
//!
//! # Example
//! The following example generates a client library for the Swagger Petstore
//! API, then makes an syncronous blocking call to create a pet.
//!
//! ```ignore
//! #[auto_api::gen_api("file://openapi/petstore.json")]
//! pub mod petstore_api {}
//!
//! fn main() {
//!     // Generate a default client with default parameters
//!     let petstore_client = petstore_api::Client::default();
//!
//!     // Make a blocking request and send it
//!     let response = petstore_client.add_pet();
//! }
//! ```

pub use auto_api_core::{client, error};
pub use auto_api_macros::gen_api;

/// All dependencies our macro needs at runtime will be put in this crate, so
/// they are accessible without the user of our crate needing them installed
/// aswell.
///
/// This module should not be used by end users
#[doc(hidden)]
pub mod __private {
    pub use {reqwest, serde};
}
