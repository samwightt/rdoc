use color_eyre::{eyre::Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

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

    // TODO: Implement search using our own parser
    println!(
        "{} Search not yet implemented - working on parser",
        "ℹ".yellow().bold()
    );

    Ok(())
}
