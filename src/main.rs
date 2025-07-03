mod cli;
mod download;

use clap::Parser;
use cli::{Cli, Commands};
use download::{download_multi, download_multi_resume, download_single, download_single_resume};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.commands {
        Commands::Single { resume, file } => {
            if resume.is_none_or(|res| !res) {
                download_single(&file)?;
            } else {
                download_single_resume(&file)?;
            }
        }
        Commands::Multi { resume, files } => {
            if resume.is_none_or(|res| !res) {
                download_multi(&files)?;
            } else {
                download_multi_resume(&files)?;
            }
        }
    }

    Ok(())
}
