use crate::{
    db::ResumeDb,
    download::{Download, DownloadStatus, download_multi, download_single},
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    sync::mpsc::{Receiver, SyncSender, sync_channel},
};

#[derive(Debug)]
pub struct DownloadManager {
    db: ResumeDb,
    queue: (SyncSender<Download>, Receiver<Download>),
}

impl DownloadManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            db: ResumeDb::new()?,
            queue: sync_channel(10),
        })
    }

    pub fn download(
        &self,
        urls: &Vec<String>,
        cookie: Option<String>,
        headers: Option<Vec<String>>,
        file_path: Option<Vec<String>>,
        file_name: Option<Vec<String>>,
        multi: bool,
    ) -> Result<(), Box<IoError>> {
        if !multi {
            if let Err(e) = download_single(
                urls[0].clone(),
                match file_path {
                    Some(file_path) if file_path.len() > 0 => Some(file_path[0].clone()),
                    _ => None,
                },
                match file_name {
                    Some(file_name) if file_name.len() > 0 => Some(file_name[0].clone()),
                    _ => None,
                },
                cookie,
                headers,
                None,
            ) {
                eprintln!("Download failed! You can try again, or try the `resume` subcommand.");
                eprintln!("{e}");
            };
        } else {
            if let Err(e) = download_multi(&urls, file_path, file_name, cookie, headers) {
                eprintln!(
                    "One or more downloads failed! You can try again, or try the `resume` command."
                );
                eprintln!("{e}");
            };
        }

        Ok(())
    }

    pub fn resume_download(
        &self,
        urls: &Vec<String>,
        cookie: Option<String>,
        headers: Option<Vec<String>>,
        multi: bool,
    ) -> Result<(), Box<dyn Error>> {
        if !multi {
            let url = urls[0].clone();

            let mut download = self.db.get_resume(&url)?;

            download.execute()?;
        }

        Ok(())
    }
}
