#![allow(dead_code)]

use std::collections::BTreeMap;

use auto_api_core::error::MacroError;
use openapiv3::ReferenceOr;

pub type PathMap<T> = BTreeMap<Path, ReferenceOr<T>>;
pub type TypePathMap = PathMap<()>; // TODO:

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(Vec<String>);

impl Path {
    pub fn from_ref(reference: &str) -> Result<Self, MacroError> {
        if !reference.starts_with("#") {
            return Err(MacroError::UnimplementedFeature(
                "non-local references are unsupported".to_string(),
            ));
        }

        // Splitting the reference into parts
        let parts = reference
            .split("/")
            .skip(1)
            .map(|it| it.to_string())
            .collect::<Vec<String>>();

        Ok(Self(parts))
    }

    pub fn push(mut self, part: &str) -> Self {
        self.0.push(part.to_string());
        self
    }
}

pub fn expand_reference<'a, T>(
    index: &'a PathMap<T>,
    reference: &'a ReferenceOr<T>,
) -> Result<&'a T, MacroError> {
    match reference {
        ReferenceOr::Item(it) => Ok(it),
        ReferenceOr::Reference { reference } => {
            let path = Path::from_ref(&reference)?;
            match index.get(&path) {
                Some(it) => expand_reference(index, it),
                None => Err(MacroError::InvalidReference(reference.to_string())),
            }
        }
    }
}
