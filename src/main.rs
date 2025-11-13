use clap::Parser;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::{fs, io};

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
        if path.is_file() {
            let _count = grep_file(path.as_path(), &args.pattern);
        }
    }

    Ok(())
}

fn grep_file(file_path: &Path, pattern: &str) -> usize {
    let mut count = 0;
    if let Ok(lines) = read_lines(file_path) {
        for line in lines.map_while(Result::ok) {
            if line.contains(pattern) {
                println!("{}: {}", file_path.to_string_lossy(), line);
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
