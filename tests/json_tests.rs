use json_rs::json::*;

#[test]
fn test_parse_json_number_valid() {
    let cases = vec![
        ("42", 42.0, ""),
        ("-3.14", -3.14, ""),
        ("0.001", 0.001, ""),
        ("1e6", 1e6, ""),
        ("-2.5E-3", -0.0025, ""),
        ("123.456e+2", 12345.6, ""),
        ("12a", 12.0, "a"),
        ("7.89xyz", 7.89, "xyz"),
        ("-0.5abc", -0.5, "abc"),
    ];

    for (input, expected_value, expected_remaining) in cases {
        let result = parse_json_number(input);
        assert!(
            result.is_ok(),
            "Parsing '{}' failed with error: {:?}",
            input,
            result
        );
        let (remaining, json_number) = result.unwrap();
        assert_eq!(
            remaining, expected_remaining,
            "Input '{}' was not parsed correctly. Expected remaining '{}', got '{}'",
            input, expected_remaining, remaining
        );
        match json_number {
            JSONObject::Number(n) => assert!(
                (n - expected_value).abs() < f64::EPSILON,
                "Parsed value {} does not match expected {}",
                n,
                expected_value
            ),
            _ => panic!("Parsed value is not a JSONObject::Number"),
        }
    }
}

#[test]
fn test_parse_json_number_invalid() {
    let cases = vec!["abc", "--5", "..12", ""];

    for input in cases {
        let result = parse_json_number(input);
        assert!(
            result.is_err(),
            "Invalid input '{}' was parsed successfully: {:?}",
            input,
            result
        );
    }
}

#[test]
fn test_parse_json_string() {
    let cases = vec![
        (r#"simple"#, "simple"),
        (r#"hello \"world\""#, "hello \\\"world\\\""),
        (r#"line\nbreak"#, "line\\nbreak"),
        (r#"tab\tindent"#, "tab\\tindent"),
        (r#"backslash\\test"#, "backslash\\\\test"),
        (
            r#"mix \" of \\ all \n escapes"#,
            "mix \\\" of \\\\ all \\n escapes",
        ),
        (
            r#"quote: \" and backslash: \\"#,
            "quote: \\\" and backslash: \\\\",
        ),
    ];

    for (input, expected) in cases {
        let formatted = format!("\"{}\"", input);
        let result = parse_json_string(&formatted);
        assert!(
            result.is_ok(),
            "Parsing '{}' failed with error: {:?}",
            input,
            result
        );
        let (remaining, json_string) = result.unwrap();
        assert_eq!(remaining, "", "Input '{}' was not fully consumed", input);
        match json_string {
            JSONObject::String(s) => {
                assert_eq!(
                    s, expected,
                    "Parsed value '{}' does not match expected '{}'",
                    s, expected
                );
            }
            _ => panic!("Parsed value is not a JSONObject::String"),
        }
    }
}

#[test]
fn test_parse_json_null() {
    let result = parse_json_null("null");
    assert!(result.is_ok(), "Expected Ok for 'null'");
    let (remaining, value) = result.unwrap();
    assert_eq!(remaining, "", "Expected no remaining input");
    match value {
        JSONObject::Null => {}
        _ => panic!("Expected JSONObject::Null"),
    }

    let invalid_inputs = vec!["nul", "NULL", "nill", "none", "Null", "nan"];
    for input in invalid_inputs {
        assert!(
            parse_json_null(input).is_err(),
            "Expected error for '{}'",
            input
        );
    }
}

#[test]
fn test_parse_json_bool() {
    let true_result = parse_json_bool("true");
    assert!(true_result.is_ok(), "Expected Ok for 'true'");
    let (remaining, value) = true_result.unwrap();
    assert_eq!(remaining, "", "Expected no remaining input");
    assert_eq!(value, JSONObject::Bool(true));

    let false_result = parse_json_bool("false");
    assert!(false_result.is_ok(), "Expected Ok for 'false'");
    let (remaining, value) = false_result.unwrap();
    assert_eq!(remaining, "", "Expected no remaining input");
    assert_eq!(value, JSONObject::Bool(false));

    let invalid_inputs = vec!["TRUE", "False", "truth", "fals"];
    for input in invalid_inputs {
        assert!(
            parse_json_bool(input).is_err(),
            "Expected error for '{}'",
            input
        );
    }
}

#[test]
fn test_parse_json_array() {
    let cases = vec![
        (
            "[1, 2, 3]",
            JSONObject::Array(vec![
                JSONObject::Number(1.0),
                JSONObject::Number(2.0),
                JSONObject::Number(3.0),
            ]),
        ),
    ];
    for (input, expected) in cases {
        let result = parse_json_array(input);
        assert!(
            result.is_ok(),
            "Parsing '{}' failed with error: {:?}",
            input,
            result
        );
        let (remaining, parsed) = result.unwrap();
        assert_eq!(remaining, "", "Expected no remaining input");
        assert_eq!(parsed, expected, "Parsed result does not match expected");
    }
}

#[test]
fn test_parse_json_value_all_cases() {
    let cases = vec![
        ("null", JSONObject::Null),
        ("true", JSONObject::Bool(true)),
        ("false", JSONObject::Bool(false)),
        ("42", JSONObject::Number(42.0)),
        ("-3.14", JSONObject::Number(-3.14)),
        (r#""hello""#, JSONObject::String("hello".to_string())),
        (
            "[true, null, 5]",
            JSONObject::Array(vec![
                JSONObject::Bool(true),
                JSONObject::Null,
                JSONObject::Number(5.0),
            ]),
        ),
        (
            r#"{"a": 1, "b": false}"#,
            JSONObject::Map({
                let mut m = Vec::new();
                m.push(("a".to_string(), JSONObject::Number(1.0)));
                m.push(("b".to_string(), JSONObject::Bool(false)));
                m
            }),
        ),
    ];

    for (input, expected) in cases {
        let result = parse_json_value(input);
        assert!(
            result.is_ok(),
            "Parsing '{}' failed with error: {:?}",
            input,
            result
        );
        let (remaining, parsed) = result.unwrap();
        assert_eq!(remaining, "", "Expected no remaining input for '{}'", input);
        assert_eq!(
            parsed, expected,
            "Parsed value does not match expected for '{}'",
            input
        );
    }
}
