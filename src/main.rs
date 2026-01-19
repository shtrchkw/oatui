mod model;
mod parser;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: oatui <openapi-file>");
        std::process::exit(1);
    }

    let file_path = &args[1];

    match parser::parse_file(file_path) {
        Ok(spec) => {
            println!("Title: {}", spec.title);
            println!("Version: {}", spec.version);
            if let Some(desc) = &spec.description {
                println!("Description: {}", desc);
            }
            println!("\nEndpoints ({}):", spec.endpoints.len());
            for endpoint in &spec.endpoints {
                println!("  {} {}", endpoint.method, endpoint.path);
                if let Some(summary) = &endpoint.summary {
                    println!("    Summary: {}", summary);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:#}", e);
            std::process::exit(1);
        }
    }
}
