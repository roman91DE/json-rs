use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{escaped, tag, take_while1},
    character::complete::{char, multispace0, one_of},
    multi::separated_list0,
    number::complete::recognize_float,
    sequence::{delimited, separated_pair},
};

#[derive(Debug, Clone, PartialEq)]
pub enum JSONObject {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSONObject>),
    Map(HashMap<String, JSONObject>),
}

pub fn parse_json_null(input: &str) -> IResult<&str, JSONObject> {
    let mut parser = tag("null");
    let result = parser.parse(input);
    match result {
        Ok((rest, _)) => Ok((rest, JSONObject::Null)),
        Err(e) => Err(e),
    }
}

pub fn parse_json_bool(input: &str) -> IResult<&str, JSONObject> {
    let mut parser = alt((tag("true"), tag("false")));
    let result = parser.parse(input);
    match result {
        Ok((rest, parsed)) => Ok((rest, JSONObject::Bool(parsed == "true"))),
        Err(e) => Err(e),
    }
}

pub fn parse_json_number(input: &str) -> IResult<&str, JSONObject> {
    let mut parser = |x| recognize_float(x);
    let result = parser.parse(input);
    match result {
        Ok((rest, parsed)) => Ok((
            rest,
            JSONObject::Number(parsed.parse().expect("Error during Float Parsing")),
        )),
        Err(e) => Err(e),
    }
}
fn parse_json_string(input: &str) -> IResult<&str, JSONObject> {
    let mut parser = delimited(
        tag("\""),
        escaped(
            take_while1(|c| c != '\\' && c != '"'),
            '\\',
            one_of("\"nrt\\"),
        ),
        tag("\""),
    );
    let result = parser.parse(input);
    match result {
        Ok((rest, parsed)) => Ok((rest, JSONObject::String(parsed.to_string()))),
        Err(e) => Err(e),
    }
}

fn parse_json_array(input: &str) -> IResult<&str, JSONObject> {
    let elements = separated_list0(
        delimited(multispace0, char(','), multispace0),
        parse_json_value,
    );
    let mut array_parser = delimited(
        delimited(multispace0, char('['), multispace0),
        elements,
        delimited(multispace0, char(']'), multispace0),
    );

    let result = array_parser.parse(input);
    match result {
        Ok((rest, parsed)) => Ok((rest, JSONObject::Array(parsed))),
        Err(e) => Err(e),
    }
}

fn parse_json_map(input: &str) -> IResult<&str, JSONObject> {
    let key_value = separated_pair(
        delimited(multispace0, parse_json_string, multispace0),
        delimited(multispace0, char(':'), multispace0),
        parse_json_value,
    );

    let map_contents = separated_list0(delimited(multispace0, char(','), multispace0), key_value);

    let mut full_parser = delimited(
        delimited(multispace0, char('{'), multispace0),
        map_contents,
        delimited(multispace0, char('}'), multispace0),
    );

    let result = full_parser.parse(input);

    match result {
        Ok((rest, parsed)) => {
            let mut map = HashMap::new();
            for (k, v) in parsed {
                if let JSONObject::String(key) = k {
                    map.insert(key, v);
                } else {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            }
            Ok((rest, JSONObject::Map(map)))
        }
        Err(e) => Err(e),
    }
}

pub fn parse_json_value(input: &str) -> IResult<&str, JSONObject> {
    delimited(
        multispace0,
        alt((
            parse_json_null,
            parse_json_bool,
            parse_json_number,
            parse_json_string,
            parse_json_array,
            parse_json_map,
        )),
        multispace0,
    )
    .parse(input)
}

impl Display for JSONObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            JSONObject::Null => write!(f, "null"),
            JSONObject::Bool(b) => write!(f, "{}", *b),
            JSONObject::Number(n) => write!(f, "{}", n),
            JSONObject::String(s) => write!(f, "\"{}\"", s),
            JSONObject::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            JSONObject::Map(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            (r#""simple""#, "simple"),
            (r#""hello \"world\"""#, "hello \\\"world\\\""),
            (r#""line\nbreak""#, "line\\nbreak"),
            (r#""tab\tindent""#, "tab\\tindent"),
            (r#""backslash\\test""#, "backslash\\\\test"),
            (
                r#""mix \" of \\ all \n escapes""#,
                "mix \\\" of \\\\ all \\n escapes",
            ),
            (
                r#""quote: \" and backslash: \\""#,
                "quote: \\\" and backslash: \\\\",
            ),
        ];

        for (input, expected) in cases {
            let result = super::parse_json_string(input);
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
        let cases = vec![(
            "[1, 2, 3]",
            JSONObject::Array(vec![
                JSONObject::Number(1.0),
                JSONObject::Number(2.0),
                JSONObject::Number(3.0),
            ]),
        )];
        for (input, expected) in cases {
            let result = super::parse_json_array(input);
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
                    let mut m = HashMap::new();
                    m.insert("a".to_string(), JSONObject::Number(1.0));
                    m.insert("b".to_string(), JSONObject::Bool(false));
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
}
