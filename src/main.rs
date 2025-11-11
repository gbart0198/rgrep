use clap::Parser;
use std::borrow::Cow;
use std::fs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory to search, e.g '.' for current directory, './app' for app directory
    directory_name: String,

    /// The pattern to search against
    pattern: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut results: Vec<_> = Vec::<String>::new();

    for entry in fs::read_dir(args.directory_name)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            results.push(path.to_str().unwrap().into());
        }
    }

    println!("{:?}", results);

    Ok(())
}
