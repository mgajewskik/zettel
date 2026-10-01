use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use zettel::commands;
use zettel::output::Format;

#[derive(Parser, Debug)]
#[command(
    name = "zettel",
    version,
    about = "Obsidian-compatible CLI for wikilinks (links, backlinks, unresolved, find)",
    long_about = None
)]
struct Cli {
    /// Path to the Obsidian vault root (overrides ZETTEL_VAULT)
    #[arg(long, global = true, value_name = "PATH")]
    vault: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = Format::Text)]
    format: Format,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List outbound links from a note
    Links {
        /// Note id, alias, title, stem, or path
        note: String,
    },
    /// List notes that link to a note
    Backlinks {
        /// Note id, alias, title, stem, or path
        note: String,
    },
    /// List broken (unresolved) links
    Unresolved {
        /// Optional path prefixes to restrict which source notes are scanned
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// Check whether a note resolves (exit 1 if not found)
    Exists {
        /// Note id, alias, title, stem, or path
        query: String,
    },
    /// Search notes by id, title, alias, or content
    Find {
        /// Search query
        query: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Links { note } => commands::cmd_links(cli.vault.clone(), cli.format, note),
        Commands::Backlinks { note } => {
            commands::cmd_backlinks(cli.vault.clone(), cli.format, note)
        }
        Commands::Unresolved { paths } => {
            commands::cmd_unresolved(cli.vault.clone(), cli.format, paths)
        }
        Commands::Exists { query } => commands::cmd_exists(cli.vault.clone(), cli.format, query),
        Commands::Find { query } => commands::cmd_find(cli.vault.clone(), cli.format, query),
    };

    match result {
        Ok(status) => status.into(),
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_status())
        }
    }
}
