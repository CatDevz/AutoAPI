use fancy_regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // This regex is used to split a camelCase, PascalCase, snake_case or SCREAMING_SNAKE_CASE
    // string. Usage: https://regex101.com/r/mJW2yk/1
    static ref CASING_SPLIT_REGEX: Regex = {
        Regex::new(r"(?>[A-Z]?)[a-z0-9]+|[A-Z]+").unwrap()
    };
}

/// Converts the casing of the inputed value from camelCase, PascalCase or
/// SCREAMING_SNAKE_CASE to snake_case.
pub fn convert_casing_to_snake(original: &str) -> String {
    CASING_SPLIT_REGEX
        .captures_iter(&original)
        .filter_map(|it| it.ok())
        .filter_map(|it| it.get(0))
        .map(|it| it.as_str())
        .collect::<Vec<&str>>()
        .join("_")
        .to_lowercase()
}

/// Converts the casing of the inputed value from camelCase, snake_case or
/// SCREAMING_SNAKE_CASE to PascalCase
pub fn convert_casing_to_pascal(original: &str) -> String {
    CASING_SPLIT_REGEX
        .captures_iter(&original)
        .filter_map(|it| it.ok())
        .filter_map(|it| it.get(0))
        .map(|it| it.as_str())
        .map(|it| {
            if it.len() > 1 {
                let (head, tail) = it.split_at(1);
                let mut res = String::new();
                res.push_str(&head.to_uppercase());
                res.push_str(&tail.to_lowercase());
                res
            } else {
                it.to_uppercase()
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::{convert_casing_to_pascal, convert_casing_to_snake};

    #[test]
    fn successfully_convert_casing_to_snake() {
        assert_eq!(&convert_casing_to_snake("snake_case"), "snake_case");
        assert_eq!(&convert_casing_to_snake("SKRM_SNEK_CASE"), "skrm_snek_case");
        assert_eq!(&convert_casing_to_snake("camelCase"), "camel_case");
        assert_eq!(&convert_casing_to_snake("PascalCase"), "pascal_case");
    }

    #[test]
    fn successfully_convert_casing_to_pascal() {
        assert_eq!(&convert_casing_to_pascal("snake_case"), "SnakeCase");
        assert_eq!(&convert_casing_to_pascal("SKRM_SNEK_CASE"), "SkrmSnekCase");
        assert_eq!(&convert_casing_to_pascal("camelCase"), "CamelCase");
        assert_eq!(&convert_casing_to_pascal("PascalCase"), "PascalCase");
    }
}
