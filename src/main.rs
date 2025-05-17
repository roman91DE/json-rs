mod json;

use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        // Read from file
        match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", args[1], e);
                std::process::exit(1);
            }
        }
    } else {
        // Read from stdin
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
        buffer
    };

    match json::parse_json_value(&input) {
        Ok((rest, json_obj)) => {
            if !rest.trim().is_empty() {
                eprintln!("Warning: Unparsed input remaining: '{}'", rest);
            }
            println!("Valid JSON:");
            println!("{}", json_obj);
        }
        Err(e) => {
            eprintln!("Invalid JSON: {:?}", e);
            std::process::exit(1);
        }
    }
}
