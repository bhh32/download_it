mod cli;
mod download;

use clap::Parser;
use cli::{Cli, Commands};
use download::{download_multi, download_multi_resume, download_single, download_single_resume};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.commands {
        Commands::Single {
            resume,
            cookie,
            header_args,
            url,
            file_path,
            file_name,
        } => {
            if resume.is_none_or(|res| !res) {
                download_single(url, file_path, file_name, cookie, header_args, None)?;
            } else {
                download_single_resume(&url)?;
            }
        }
        Commands::Multi {
            resume,
            urls,
            cookie,
            header_args,
            file_path,
            file_names,
        } => {
            if resume.is_none_or(|res| !res) {
                download_multi(&urls, file_path, file_names, cookie, header_args)?;
            } else {
                download_multi_resume(&urls)?;
            }
        }
    }

    Ok(())
}
