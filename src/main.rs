use clap::Parser;
use std::fs;
use std::path::Path;

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

    for entry in fs::read_dir(args.directory_name)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && !grep_file(path.as_path(), &args.pattern) {
            println!("Pattern not found in {}", path.to_str().unwrap());
        }
    }

    Ok(())
}

fn grep_file(file_path: &Path, pattern: &str) -> bool {
    let contents = std::fs::read_to_string(file_path);

    if let Ok(contents) = contents
        && let Some(found) = contents.find(pattern)
    {
        println!(
            "Found match in file {} at index: {}",
            file_path.to_str().unwrap(),
            found
        );
        return true;
    }
    false
}
