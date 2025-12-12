use clap::Parser;
use colored::*;
use std::fmt::Display;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::{fs, io};

#[derive(Debug)]
struct FileSearchResult {
    file_name: String,
    search_results: Vec<SearchResult>,
}

#[derive(Debug)]
struct SearchResult {
    line_number: u32,
    match_text: String,
}

impl Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.line_number, self.match_text)
    }
}

impl Display for FileSearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut results = String::new();

        for result in &self.search_results {
            results.push_str(&format!("{}\n", result));
        }

        write!(f, "{}\n{}", self.file_name, results)
    }
}

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
    let mut total_matches = Vec::new();

    for entry in fs::read_dir(args.directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.to_string_lossy();
            if args.file != "*" {
                if file_name.contains(&args.file) {
                    if let Some(results) = grep_file(&path, &args.pattern) {
                        total_matches.push(results);
                    }
                }
            } else {
                if let Some(results) = grep_file(&path, &args.pattern) {
                    total_matches.push(results);
                }
            }
        }
    }

    for found in &total_matches {
        println!("{}", found);
    }

    Ok(())
}

fn grep_file(file_path: &Path, pattern: &str) -> Option<FileSearchResult> {
    let mut line_number = 0;
    let mut matches = Vec::new();
    if let Ok(lines) = read_lines(file_path) {
        for line in lines.map_while(Result::ok) {
            if let Some(index) = line.find(pattern) {
                let before = &line[..index];
                let after = &line[index + pattern.len()..];
                let match_string = format!("{}{}{}", before, pattern.red().bold(), after);
                matches.push(SearchResult {
                    match_text: match_string,
                    line_number,
                });
            }
            line_number += 1;
        }
    }
    if matches.is_empty() {
        None
    } else {
        Some(FileSearchResult {
            file_name: file_path.to_str().unwrap_or("").into(),
            search_results: matches,
        })
    }
}

fn read_lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}
