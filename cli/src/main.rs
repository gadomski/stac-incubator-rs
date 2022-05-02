use clap::{Parser, Subcommand};
use stac::PathBufHref;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Downloads objects and assets.
    Download {
        /// Href of the STAC object.
        href: String,

        /// The output directory into which the object and assets will be downloaded.
        outdir: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Download { href, outdir }) => {
            stac_cli::download::download_item(PathBufHref::from(href).into(), PathBuf::from(outdir))
                .await
                .unwrap()
        }
        None => {}
    }
}
