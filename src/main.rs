use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use zettel::commands::{self, CheckRule, CheckScope, FindMode};
use zettel::output::Format;

#[derive(Parser, Debug)]
#[command(
    name = "zettel",
    version,
    about = "Obsidian-compatible CLI for wikilinks (links, backlinks, unresolved, find, check-links)",
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
        /// Optional path filters: only report broken links whose *source* matches
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,

        /// Include notes under archive/ as sources (excluded by default)
        #[arg(long)]
        include_archive: bool,
    },
    /// Check whether a note resolves (exit 1 if not found)
    Exists {
        /// Note id, alias, title, stem, or path
        query: String,
    },
    /// Search notes by id, path, title, alias, or content
    Find {
        /// Search query
        query: String,

        /// Search mode (default: auto = structural first, content fallback)
        #[arg(long, value_enum, default_value_t = FindMode::Auto)]
        mode: FindMode,

        /// Include notes under archive/ (excluded by default)
        #[arg(long)]
        include_archive: bool,
    },
    /// Enforce vault link conventions
    CheckLinks {
        /// Rule to enforce
        #[arg(long, value_enum)]
        rule: CheckRule,

        /// Permanent note scope (default: root = vault-root *.md only)
        #[arg(long, value_enum, default_value_t = CheckScope::Root)]
        scope: CheckScope,

        /// Optional path filters: only check matching *permanent* sources
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,

        /// Include notes under archive/ as sources (excluded by default; no-op for root scope)
        #[arg(long)]
        include_archive: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Links { note } => commands::cmd_links(cli.vault.clone(), cli.format, note),
        Commands::Backlinks { note } => {
            commands::cmd_backlinks(cli.vault.clone(), cli.format, note)
        }
        Commands::Unresolved {
            paths,
            include_archive,
        } => commands::cmd_unresolved(cli.vault.clone(), cli.format, paths, *include_archive),
        Commands::Exists { query } => commands::cmd_exists(cli.vault.clone(), cli.format, query),
        Commands::Find {
            query,
            mode,
            include_archive,
        } => commands::cmd_find(cli.vault.clone(), cli.format, query, *mode, *include_archive),
        Commands::CheckLinks {
            rule,
            scope,
            paths,
            include_archive,
        } => commands::cmd_check_links(
            cli.vault.clone(),
            cli.format,
            *rule,
            *scope,
            paths,
            *include_archive,
        ),
    };

    match result {
        Ok(status) => status.into(),
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_status())
        }
    }
}
