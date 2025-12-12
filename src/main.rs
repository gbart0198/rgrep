use clap::Parser;
use colored::*;
use std::fmt::Display;
use std::fs::File;
use std::io::BufRead;
use std::os::unix::raw::time_t;
use std::path::{Path, PathBuf};
use std::{fs, io, time};

use futures::StreamExt;
use futures::stream;
use tokio;

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

    /// The number of threads to use for searching
    #[arg(short, long, default_value = "4")]
    threads: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.threads == 0 {
        return Err("Number of threads must be greater than 0".into());
    }

    let mut time_start = time::Instant::now();

    let _ = grep_multi_thread(&args).await?;

    let time_multi = time_start.elapsed().as_nanos();
    time_start = time::Instant::now();

    let _ = grep_single_thread(&args)?;
    let time_single = time_start.elapsed().as_nanos();

    println!("Time elapsed for multi-threaded search: {} ns", time_multi);
    println!(
        "Time elapsed for single-threaded search: {} ns",
        time_single
    );

    Ok(())
}

async fn grep_multi_thread(
    args: &Args,
) -> Result<Vec<FileSearchResult>, Box<dyn std::error::Error>> {
    let mut items_to_search: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&args.directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.to_string_lossy();
            if args.file != "*" {
                if file_name.contains(&args.file) {
                    items_to_search.push(path);
                }
            } else {
                items_to_search.push(path);
            }
        }
    }

    // Build a stream of futures (one future per file). Each future owns its PathBuf and the pattern clone.
    // Use `buffer_unordered` to run up to `args.threads` futures concurrently, then collect the results.
    let concurrency = args.threads as usize;
    let pattern = args.pattern.clone();

    let results: Vec<Option<FileSearchResult>> = futures::stream::iter(items_to_search)
        .map(|item| {
            let pat = pattern.clone();
            search_file_multi_thread(item, pat)
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    // Flatten Option and collect into total_matches
    let mut total_matches = Vec::new();
    for opt in results.into_iter().flatten() {
        total_matches.push(opt);
    }

    Ok(total_matches)
}

fn grep_single_thread(args: &Args) -> Result<Vec<FileSearchResult>, Box<dyn std::error::Error>> {
    let mut total_matches = Vec::new();
    for entry in fs::read_dir(&args.directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.to_string_lossy();
            if args.file != "*" {
                if file_name.contains(&args.file) {
                    if let Some(results) = search_file(&path, &args.pattern) {
                        total_matches.push(results);
                    }
                }
            } else {
                if let Some(results) = search_file(&path, &args.pattern) {
                    total_matches.push(results);
                }
            }
        }
    }
    Ok(total_matches)
}

// Accept an owned PathBuf so the async task owns its data (no borrowed lifetimes across await points).
async fn search_file_multi_thread(file_path: PathBuf, pattern: String) -> Option<FileSearchResult> {
    let mut line_number = 0;
    let mut matches = Vec::new();
    if let Ok(lines) = read_lines(&file_path) {
        for line in lines.map_while(Result::ok) {
            if let Some(index) = line.find(&pattern) {
                let before = &line[..index];
                let after = &line[index + pattern.len()..];
                let match_string = format!("{}{}{}", before, pattern.as_str().red().bold(), after);
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

fn search_file(file_path: &Path, pattern: &str) -> Option<FileSearchResult> {
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
