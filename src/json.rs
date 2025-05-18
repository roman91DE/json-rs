use std::fmt::{self, Display, Formatter};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_while_m_n},
    character::complete::{char, multispace0},
    combinator::{map, recognize},
    multi::{many0, separated_list0},
    number::complete::recognize_float,
    sequence::{delimited, preceded, separated_pair},
};

#[derive(Debug, Clone, PartialEq)]
pub enum JSONObject {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSONObject>),
    Map(Vec<(String, JSONObject)>),
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
pub fn parse_json_string(input: &str) -> IResult<&str, JSONObject> {
    // Parser for a single escape sequence (does not handle unicode)
    let parse_escape = preceded(
        char('\\'),
        alt((
            char('"'),
            char('\\'),
            char('/'),
            char('b'),
            char('f'),
            char('n'),
            char('r'),
            char('t'),
            // Unicode escapes (\uXXXX)
            map(
                preceded(
                    char('u'),
                    take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit()),
                ),
                |_| 'u',
            ),
        )),
    );

    // Parser for a single string fragment (either normal or escaped)
    let parse_fragment = alt((
        is_not("\\\""), // normal string chars except backslash and quote
        recognize(parse_escape),
    ));

    let parse_string_content = map(many0(parse_fragment), |fragments: Vec<&str>| {
        fragments.concat()
    });

    let mut parser = delimited(tag("\""), parse_string_content, tag("\""));
    let result = parser.parse(input);
    match result {
        Ok((rest, parsed)) => Ok((rest, JSONObject::String(parsed))),
        Err(e) => Err(e),
    }
}

pub fn parse_json_array(input: &str) -> IResult<&str, JSONObject> {
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
            let mut vec = Vec::new();
            for (k, v) in parsed {
                if let JSONObject::String(key) = k {
                    vec.push((key, v));
                } else {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            }
            Ok((rest, JSONObject::Map(vec)))
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
            JSONObject::Map(vec) => {
                let pairs: Vec<String> = vec
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}
