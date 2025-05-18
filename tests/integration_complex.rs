use json_rs::json::parse_json_value;
use std::fs;

#[test]
fn test_parse_complex_json_file() {
    let content =
        fs::read_to_string("tests/test_cases/test.json").expect("Failed to read test.json");
    let result = parse_json_value(&content);
    assert!(
        result.is_ok(),
        "Parsing test.json failed with error: {:?}",
        result
    );
    let (remaining, _) = result.unwrap();
    assert!(
        remaining.trim().is_empty(),
        "Unparsed input remaining: '{}'",
        remaining
    );
}
