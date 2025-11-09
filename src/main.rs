use clap::{Parser, Subcommand};
use color_eyre::Result;

mod commands;
mod search_index;

/// A CLI tool for searching generated Rust documentation
#[derive(Parser)]
#[command(name = "rdoc")]
#[command(about = "Search generated Rust documentation", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan rustdocs for a specific symbol
    #[command(about = "Search for a symbol in generated rustdocs")]
    Scan {
        /// The symbol to search for (e.g., "Result", "Vec", "HashMap")
        #[arg(value_name = "SYMBOL")]
        symbol: String,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan { symbol }) => {
            commands::scan::execute(&symbol)?;
        }
        None => {
            // When no subcommand is provided, show help
            Cli::parse_from(&["rdoc", "--help"]);
        }
    }

    Ok(())
}
