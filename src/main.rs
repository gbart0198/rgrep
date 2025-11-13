use clap::Parser;
use colored::*;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::{fs, io};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The pattern to search against
    pattern: String,

    /// The directory to search, e.g '.' for current directory, './app' for app directory
    #[arg(short, long, default_value = ".")]
    directory: String,

    /// The file pattern to search against
    #[arg(short, long, default_value = "*")]
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut total = 0;

    for entry in fs::read_dir(args.directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.to_string_lossy();
            if args.file != "*" {
                if file_name.contains(&args.file) {
                    total += grep_file(&path, &args.pattern);
                }
            } else {
                total += grep_file(&path, &args.pattern);
            }
        }
    }

    println!("Total matches found: {}", total);

    Ok(())
}

fn grep_file(file_path: &Path, pattern: &str) -> usize {
    let mut count = 0;
    if let Ok(lines) = read_lines(file_path) {
        for line in lines.map_while(Result::ok) {
            if let Some(index) = line.find(pattern) {
                let before = &line[..index];
                let after = &line[index + pattern.len()..];
                println!("{}{}{}", before, pattern.red().bold(), after);
                count += 1;
            }
        }
    }
    count
}

fn read_lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}
