use curl::easy::Easy;
use dirs::download_dir;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Download {
    pub url: String,
    pub file_name: String,
    pub file_path: String,
    pub progress_bar: Arc<ProgressBar>,
    pub status: DownloadStatus,
    pub error: Option<String>,
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
}

#[derive(Debug)]
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

pub fn download_single_resume(_url: &str) -> Result<(), Box<dyn Error>> {
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
    file_path: Option<String>,
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
                Some(file_names[idx].clone())
            } else {
                None
            };
            let file_path = file_path.clone();
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
                    Err(e) => {}
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

pub fn download_multi_resume(_urls: &Vec<String>) -> Result<(), Box<dyn Error>> {
    Ok(())
}
