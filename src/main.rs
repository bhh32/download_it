mod cli;
pub mod db;
mod download;
mod download_manager;

use std::fmt::Debug;

use crate::download_manager::DownloadManager;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let manager = DownloadManager::new()?;

    match args.commands {
        Commands::Single {
            cookie,
            header_args,
            url,
            file_path,
            file_name,
        } => {
            let mut urls = Vec::new();
            let mut file_names = Some(Vec::new());

            urls.push(url);
            if let Some(file_name) = file_name {
                file_names.as_mut().unwrap().push(file_name);
            };

            let mut cookies = Some(Vec::new());
            if let Some(cookie) = cookie {
                cookies.as_mut().unwrap().push(cookie);
            };

            manager.download(&urls, file_path, file_names, cookies, header_args, false)?;
        }
        Commands::Multi {
            urls,
            cookie,
            header_args,
            file_path,
            file_names,
        } => {
            // Temp workaround
            let mut file_paths = Some(Vec::<String>::new());
            if let Some(file_path) = file_path.clone() {
                file_paths.as_mut().unwrap().push(file_path.clone());
            };

            manager.download(&urls, cookie, header_args, file_paths, file_names, true)?;
        }
        Commands::Resume { multi, .. } => {
            if multi.is_none() || !multi.unwrap() {
                //download_single_resume(&url)?;
            } else {
                //download_download_multi(&url)?;
            }
        }
    }

    Ok(())
}
