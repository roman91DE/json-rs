use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use nom::Parser;
use nom::{IResult, number::complete::recognize_float};
use nom::{
    bytes::complete::{escaped, tag, take_while1},
    character::complete::one_of,
    sequence::delimited,
};

use nom::branch::alt;

#[derive(Debug, Clone)]
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
    todo!()
}

fn parse_json_map(input: &str) -> IResult<&str, JSONObject> {
    todo!()
}

impl Display for JSONObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            JSONObject::Null => write!(f, "null"),
            JSONObject::Bool(b) => write!(f, "{}", *b==true),
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
}

