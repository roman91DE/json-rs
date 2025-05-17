mod json;

// {
//     name: "henry",
//     age: 42,
//     jobs: ["farmer, programmer"],
//     has_swag: true,
//     friends: null
// }

fn main() {
    let input = r#"
    {
        "name": "henry",
        "age": 42,
        "jobs": ["farmer", "programmer"],
        "has_swag": true,
        "friends": null
    }
    "#;

    match json::parse_json_value(input) {
        Ok((rest, json_obj)) => {
            if !rest.trim().is_empty() {
                eprintln!("Warning: Unparsed input remaining: '{}'", rest);
            }
            println!("{}", json_obj);
        }
        Err(e) => {
            eprintln!("Failed to parse JSON: {:?}", e);
        }
    }
}
