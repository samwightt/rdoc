use color_eyre::{eyre::Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

use crate::search_index::{extract_json_string, parse_search_index};
use crate::search_items::decode_crate;

/// Scan rustdocs for a specific symbol
pub fn execute(symbol: &str) -> Result<()> {
    println!(
        "{} Scanning for symbol: {}",
        "→".cyan().bold(),
        symbol.green().bold()
    );

    // Check if we're in a Rust project
    if !Path::new("Cargo.toml").exists() {
        return Err(color_eyre::eyre::eyre!(
            "No Cargo.toml found. Please run rdoc from a Rust project directory."
        ));
    }

    let search_index_path = Path::new("target/doc/search-index.js");

    // Check if docs exist, if not generate them
    if !search_index_path.exists() {
        println!(
            "{} Documentation not found. Generating with cargo doc...",
            "ℹ".blue().bold()
        );

        let output = Command::new("cargo")
            .arg("doc")
            .output()
            .wrap_err("Failed to execute cargo doc")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "cargo doc failed:\n{}",
                stderr
            ));
        }

        println!("{} Documentation generated successfully!", "✓".green().bold());
    }

    // Parse the search index
    let content = std::fs::read_to_string(search_index_path)
        .wrap_err("Failed to read search-index.js")?;

    let json_string = extract_json_string(&content);
    let crate_entries = parse_search_index(&json_string);

    // Decode all crates into search items
    let mut all_items = Vec::new();
    for entry in &crate_entries {
        let items = decode_crate(&entry.name, &entry.data);
        all_items.extend(items);
    }

    // Search for items matching the symbol (case-insensitive substring match)
    let search_term = symbol.to_lowercase();
    let results: Vec<_> = all_items
        .iter()
        .filter(|item| item.name.to_lowercase().contains(&search_term))
        .collect();

    // Display results
    if results.is_empty() {
        println!("{} No results found for \"{}\"", "✗".red().bold(), symbol);
    } else {
        println!(
            "\n{} Found {} result{} for \"{}\":\n",
            "✓".green().bold(),
            results.len(),
            if results.len() == 1 { "" } else { "s" },
            symbol
        );

        for item in results {
            let type_str = format!("{:?}", item.item_type);
            println!(
                "  {} ({}) in {}",
                item.name.cyan(),
                type_str.yellow(),
                item.crate_name.dimmed()
            );
            if !item.path.is_empty() {
                println!("    at {}", item.path.dimmed());
            }
        }
    }

    Ok(())
}
