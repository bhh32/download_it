use curl::easy::Easy;
use dirs::download_dir;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Download {
    pub url: String,
    pub file_name: String,
    pub file_path: String,
    #[serde(skip, default = "default_progress_bar")]
    pub progress_bar: Arc<ProgressBar>,
    pub status: DownloadStatus,
    pub error: Option<String>,
}

fn default_progress_bar() -> Arc<ProgressBar> {
    let progress_bar = Arc::new(ProgressBar::new(0));

    progress_bar
}

impl Download {
    pub fn new(
        url: String,
        file_name: Option<String>,
        file_path: Option<String>,
        multi_progress: Option<&MultiProgress>,
    ) -> Self {
        let file_name = if let Some(file_name) = file_name {
            file_name
        } else {
            let file_name = url.clone();
            let file_name = file_name
                .split("/")
                .last()
                .expect("Should have gotten file name");
            file_name.to_string()
        };

        let file_path = if let Some(file_path) = file_path {
            file_path
        } else {
            let downloads = download_dir()
                .expect("Couldn't get the download directory")
                .to_string_lossy()
                .to_string();

            downloads.clone()
        };

        let progress_bar = ProgressBar::new(0);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Could not create a default ProgressBar template")
            .progress_chars("#>-")
        );

        let progress_bar = if let Some(multi_progress) = multi_progress {
            Arc::new(multi_progress.add(progress_bar))
        } else {
            Arc::new(progress_bar)
        };

        Self {
            url,
            file_name,
            file_path,
            progress_bar,
            status: DownloadStatus::Pending,
            error: None,
        }
    }

    pub fn execute(
        &mut self,
        cookie: Option<String>,
        headers: Option<Vec<String>>,
    ) -> Result<(), Box<dyn Error>> {
        // Create the download file path
        let file_path = Path::new(&self.file_path);
        if !file_path.exists() {
            std::fs::create_dir_all(file_path)?;
        }

        let file_path = &format!("{}/{}", self.file_path, self.file_name);

        // Create the file that we're downloading into
        let file = File::create(file_path)?;
        // Create a file_ref for thread sharing
        let file_ref = Arc::new(Mutex::new(file));

        // Create a cURL easy struct
        let mut easy = Easy::new();
        // Allow following redirects for the download
        easy.follow_location(true)?;
        // Give it the application as the User Agent
        easy.useragent("download_it/0.1.0")?;
        // Pass it the download URL
        easy.url(&self.url)?;
        // Get download progress from it
        easy.progress(true)?;

        // If a cookie file was passed, give it to the cURL struct
        if let Some(cookie) = cookie {
            easy.cookie(&cookie)?;
        }

        // If header arguments were given, pass them to the cURL struct
        if let Some(h_args) = headers {
            let mut list = curl::easy::List::new();

            for arg in h_args {
                list.append(&arg)?;
            }

            easy.http_headers(list)?;
        }

        self.status = DownloadStatus::InProgress;

        // Create a progress_bar from Self.progress_bar
        let progress_bar = Arc::clone(&self.progress_bar);
        easy.progress_function(move |dl_total, dl_now, _ul_total, _ul_now| {
            // Update the progress_bar
            if dl_total > 0.0 {
                progress_bar.set_length(dl_total as u64);
                progress_bar.set_position(dl_now as u64);
            }
            true
        })?;

        // Write the data to the file
        match easy.write_function(move |data| match file_ref.lock().unwrap().write_all(data) {
            Ok(_) => Ok(data.len()),
            Err(e) => {
                eprintln!("Error writing download to the file: {e}");
                Err(curl::easy::WriteError::Pause)
            }
        }) {
            Ok(_) => {
                self.status = DownloadStatus::Completed;
                self.progress_bar.finish();
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.status = DownloadStatus::Failed;
                self.progress_bar.finish();
            }
        };

        // Do the cURL process
        easy.perform()?;

        Ok(())
    }

    fn execute_resume(
        &mut self,
        cookie: Option<String>,
        headers: Option<Vec<String>>,
    ) -> Result<(), Box<dyn Error>> {
        // Get the file path passed in
        let file_path = &format!("{}/{}", self.file_path, self.file_name);

        // Get the resume position of the file
        let resume_from = fs::metadata(file_path)?.len();

        // Open the file in append mode
        let file = OpenOptions::new().append(true).open(file_path)?;

        let file_ref = Arc::new(Mutex::new(file));

        let mut easy = Easy::new();
        easy.follow_location(true)?;
        easy.useragent("download_it/0.1.0")?;
        easy.url(&self.url)?;
        easy.progress(true)?;

        // Set HTTP Range header for resume
        easy.range(&format!("{resume_from}-"))?;

        // Check if there was a cookie file passed in
        if let Some(cookie) = cookie {
            easy.cookie(&cookie)?;
        }

        // Check if there are headers and add them if there are
        if let Some(headers) = headers {
            let mut list = curl::easy::List::new();

            for header in headers {
                list.append(&header)?;
            }

            easy.http_headers(list)?;
        }

        self.status = DownloadStatus::InProgress;

        let progress_bar = Arc::clone(&self.progress_bar);
        easy.progress_function(move |dl_total, dl_now, _ul_total, _ul_now| {
            // Update the progress bar
            if dl_total > 0.0 {
                progress_bar.set_length(dl_total as u64);
                progress_bar.set_position(dl_now as u64);
            }
            true
        })?;

        // Write the data to the file
        match easy.write_function(move |data| match file_ref.lock().unwrap().write_all(data) {
            Ok(_) => Ok(data.len()),
            Err(e) => {
                eprintln!("Error writing download to the file: {e}");
                Err(curl::easy::WriteError::Pause)
            }
        }) {
            Ok(_) => {
                self.status = DownloadStatus::Completed;
                self.progress_bar.finish();
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.status = DownloadStatus::Failed;
                self.progress_bar.finish();
            }
        };

        // Do the cURL process
        easy.perform()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DownloadStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl PartialEq for DownloadStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pending, Self::Pending)
            | (Self::InProgress, Self::InProgress)
            | (Self::Completed, Self::Completed)
            | (Self::Failed, Self::Failed) => true,
            _ => false,
        }
    }
}

pub fn download_single_resume(
    url: &str,
    file_path: Option<String>,
    file_name: Option<String>,
    cookie: Option<String>,
    header_args: Option<Vec<String>>,
    download: Option<Download>,
) -> Result<(), Box<dyn Error>> {
    if file_path.is_none() || file_name.is_none() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Cannot resume without a file path or file name",
        )));
    }

    let mut download = if let Some(download) = download {
        download
    } else {
        Download::new(url.into(), file_name, file_path, None)
    };

    download.execute_resume(cookie, header_args)?;

    Ok(())
}

pub fn download_single(
    url: String,
    file_path: Option<String>,
    file_name: Option<String>,
    cookie: Option<String>,
    header_args: Option<Vec<String>>,
    download: Option<Download>,
) -> Result<(), Box<dyn Error>> {
    let mut download = if let Some(download) = download {
        download
    } else {
        Download::new(url, file_name, file_path, None)
    };

    download.execute(cookie, header_args)?;

    Ok(())
}

pub fn download_multi(
    urls: &Vec<String>,
    file_path: Option<Vec<String>>,
    file_names: Option<Vec<String>>,
    cookie: Option<String>,
    header_args: Option<Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    thread::scope(|s| {
        let mut threads = Vec::new();
        let multi_progress = MultiProgress::new();
        for (idx, url) in urls.into_iter().enumerate() {
            let url = url.clone();
            let cookie = cookie.clone();
            let header_args = header_args.clone();
            let file_name = if let Some(file_names) = file_names.clone() {
                if file_names.len() == urls.len() {
                    Some(file_names[idx].clone())
                } else {
                    None
                }
            } else {
                None
            };
            let file_path = if let Some(file_path) = file_path.clone() {
                if file_path.len() == 1 {
                    file_path[0].clone()
                } else if file_path.len() == urls.len() {
                    file_path[idx].clone()
                } else {
                    dirs::download_dir()
                        .expect("Download directory")
                        .to_string_lossy()
                        .to_string()
                }
            } else {
                dirs::download_dir()
                    .expect("Download directory")
                    .to_string_lossy()
                    .to_string()
            };
            let download = Download::new(
                url.clone(),
                file_name.clone(),
                Some(file_path.clone()),
                Some(&multi_progress),
            );
            threads.push(s.spawn(move || {
                match download_single(
                    url.clone(),
                    Some(file_path.clone()),
                    file_name.clone(),
                    cookie,
                    header_args,
                    Some(download),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{url} failed to download: {e}");
                    }
                };
            }));
        }

        for t in threads {
            t.join()
                .expect("There was a thread that failed to rejoin the main thread!");
        }
    });

    Ok(())
}

pub fn download_multi_resume(
    urls: &Vec<String>,
    file_paths: Option<Vec<String>>,
    file_names: Option<Vec<String>>,
    cookie: Option<String>,
    header_args: Option<Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    if file_paths.is_none() || file_names.is_none() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "You must enter at least on file path and file name to resume.",
        )));
    }

    thread::scope(|s| {
        let mut threads = Vec::new();
        let multi_progress = MultiProgress::new();
        for (idx, url) in urls.into_iter().enumerate() {
            let url = url.clone();
            let cookie = cookie.clone();
            let header_args = header_args.clone();
            let file_path = match file_paths.clone().unwrap().clone().iter().nth(idx) {
                Some(file_path) => Some(file_path.clone()),
                None => {
                    eprintln!("The file_path argument did not have a value for this download!");
                    eprintln!("Skipping download for {url}!");
                    continue;
                }
            };
            let file_name = match file_names.clone().unwrap().clone().iter().nth(idx) {
                Some(file_name) => Some(file_name.clone()),
                None => {
                    eprintln!("The file_names argument did not have a value for this download!");
                    eprintln!("Skipping download for {url}!");
                    continue;
                }
            };
            let download = Download::new(
                url.clone(),
                file_name.clone(),
                file_path.clone(),
                Some(&multi_progress),
            );
            threads.push(s.spawn(move || {
                match download_single(
                    url.clone(),
                    file_path.clone(),
                    file_name.clone(),
                    cookie,
                    header_args,
                    Some(download),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{url} failed to download: {e}");
                    }
                };
            }));
        }

        for t in threads {
            t.join()
                .expect("There was a thread that failed to rejoin the main thread!");
        }
    });

    Ok(())
}
