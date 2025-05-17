use json::JSONObject;
use std::collections::HashMap;

mod json;

// {
//     name: "henry",
//     age: 42,
//     jobs: ["farmer, programmer"]
// }

fn main() {
    let name = "henry".to_string();
    let name_obj = JSONObject::String(name);

    let age = 42.0;
    let age_obj = JSONObject::Number(age);

    let jobs = vec![
        JSONObject::String("farmer".to_string()),
        JSONObject::String("programmer".to_string()),
    ];
    let jobs_obj = JSONObject::Array(jobs);

    let mut map: HashMap<String, JSONObject> = HashMap::new();
    map.insert("name".to_string(), name_obj);
    map.insert("age".to_string(), age_obj);
    map.insert("jobs".to_string(), jobs_obj);

    let json_obj = JSONObject::Map(map);

    println!("{json_obj}");
}
