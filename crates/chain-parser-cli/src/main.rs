//! Vulcan Chain Parser CLI.
//!
//! A command-line tool for validating and inspecting KDL workflow files.

use std::env;
use std::fs;
use std::path::Path;

use vulcan_chain_parser::{ChainParserService, ImportFetcher, ParseError, Result, WorkflowContext};

/// File-based import fetcher for local workflow validation.
///
/// Resolves import URLs by extracting the filename and looking for it
/// in the base path directory.
struct FileFetcher {
    base_path: String,
}

impl FileFetcher {
    fn new(base_path: String) -> Self {
        Self { base_path }
    }
}

impl ImportFetcher for FileFetcher {
    fn fetch(&self, url: &str) -> Result<String> {
        // For local testing, treat URLs as relative file paths
        let path = if url.starts_with("https://") || url.starts_with("http://") {
            // Extract the filename from URL
            url.rsplit('/').next().unwrap_or(url)
        } else {
            url
        };

        let full_path = format!("{}/{}", self.base_path, path);
        fs::read_to_string(&full_path).map_err(|e| ParseError::FetchFailed {
            url: url.to_string(),
            reason: e.to_string(),
        })
    }
}

fn print_usage() {
    eprintln!("vulcan-parse - Validate and inspect Vulcan CI workflow files");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    vulcan-parse <workflow.kdl> [OPTIONS]");
    eprintln!();
    eprintln!("ARGS:");
    eprintln!("    <workflow.kdl>    Path to the KDL workflow file to parse");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    --base-path <dir>    Base directory for resolving imports (default: file's directory)");
    eprintln!("    --quiet              Only output errors, no success details");
    eprintln!("    --help               Print this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"--help".to_string()) {
        print_usage();
        std::process::exit(if args.contains(&"--help".to_string()) { 0 } else { 1 });
    }

    let workflow_path = &args[1];
    let quiet = args.contains(&"--quiet".to_string());

    // Determine base path for imports
    let base_path = args
        .iter()
        .position(|a| a == "--base-path")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| {
            Path::new(workflow_path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        });

    // Read the workflow file
    let content = match fs::read_to_string(workflow_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", workflow_path, e);
            std::process::exit(1);
        }
    };

    // Parse the workflow
    let fetcher = FileFetcher::new(base_path);
    let service = ChainParserService::new(fetcher);
    let context = WorkflowContext::new(uuid::Uuid::new_v4()).with_source(workflow_path.clone());

    match service.parse_without_trigger_validation(&content, &context) {
        Ok(result) => {
            if quiet {
                std::process::exit(0);
            }

            println!("Parsed workflow successfully!");
            println!();
            println!("Chain:");
            println!("  ID: {}", result.chain.id);
            println!("  Default Machine: {:?}", result.chain.default_machine);
            println!();
            println!("Fragments ({}):", result.fragments.len());

            for (i, frag) in result.fragments.iter().enumerate() {
                println!("  [{}] ID: {}", i, frag.id);
                println!("      Type: {:?}", frag.fragment_type);
                println!("      Sequence: {}", frag.sequence);

                if let Some(ref parent) = frag.parent_fragment_id {
                    println!("      Parent: {parent}");
                }

                if let Some(ref script) = frag.run_script {
                    let preview: String = script.chars().take(60).collect();
                    let preview = preview.replace('\n', " ");
                    if script.len() > 60 {
                        println!("      Script: {preview}...");
                    } else {
                        println!("      Script: {preview}");
                    }
                }

                if let Some(ref machine) = frag.machine {
                    println!("      Machine: {machine}");
                }

                if let Some(ref condition) = frag.condition {
                    println!("      Condition: {condition}");
                }

                if let Some(ref url) = frag.source_url {
                    println!("      Source: {url}");
                }

                println!();
            }
        }
        Err(e) => {
            eprintln!("Parse error in '{}': {}", workflow_path, e);
            std::process::exit(1);
        }
    }
}
