use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Download a STAC Item and its assets.
    Download {
        /// The href of the STAC Item.
        href: String,

        /// The directory that will hold the Item and its assets. If it does not
        /// exist, it will be created.
        directory: PathBuf,
    },

    /// Validate a STAC object.
    Validate { href: String },
}

#[tokio::main]
async fn main() {
    use Command::*;

    let cli = Cli::parse();
    let code = match cli.command {
        Download { href, directory } => stac_cli::download(href, directory).await,
        Validate { href } => stac_cli::validate(href).await,
    };
    std::process::exit(code);
}
