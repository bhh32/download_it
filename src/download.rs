use curl::easy::Easy;
use dirs::download_dir;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn download_single(url: &str) -> Result<(), Box<dyn Error>> {
    let downloads = match download_dir() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            #[cfg(not(windows))]
            let home = Command::new("echo")
                .arg("$HOME")
                .output()
                .expect("Should have gotten the $HOME value");

            #[cfg(windows)]
            let home = Command::new("cmd.exe")
                .args(["echo", "%USERPROFILE%"])
                .output()
                .expect("Should have returned the %USERPROFILE% value");

            let home_str = &String::from_utf8(home.stdout)
                .expect("Should have converted the command output to a String");
            let downloads = Path::new(&home_str).join("Downloads");

            downloads.to_string_lossy().to_string()
        }
    };

    let filename = url.to_string();
    let filename = filename
        .split("/")
        .last()
        .expect("Should have gotten file name");

    let file_path = format!("{downloads}/{filename}");
    let file = File::create(&file_path)?;
    let file_ref = Arc::new(Mutex::new(file));

    let mut easy = Easy::new();
    easy.follow_location(true)?;
    easy.useragent("download_it/0.1.0")?;
    easy.url(url)?;
    easy.progress(true)?;

    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    easy.progress_function(move |dl_total, dl_now, _ul_total, _ul_now| {
        if dl_total > 0.0 {
            progress_bar.set_length(dl_total as u64);
            progress_bar.set_position(dl_now as u64);
        }
        true
    })?;

    easy.write_function(move |data| match file_ref.lock().unwrap().write_all(data) {
        Ok(_) => Ok(data.len()),
        Err(e) => {
            eprintln!("Error writing download to the file: {e}");
            Err(curl::easy::WriteError::Pause)
        }
    })?;

    easy.perform()?;

    Ok(())
}

pub fn download_single_resume(_url: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn download_multi(urls: &Vec<String>) -> Result<(), Box<dyn Error>> {
    thread::scope(|s| {
        urls.iter().for_each(|url| {
            let url = url.clone();
            let mut threads = Vec::new();
            threads.push(s.spawn(move || {
                match download_single(&url) {
                    Ok(_) => println!("Download for {url} has completed!"),
                    Err(e) => eprintln!("Download for {url} has failed: {e}"),
                };
            }));

            for t in threads {
                t.join()
                    .expect("There was thread that failed to rejoin the main thread!");
            }
        });
    });

    Ok(())
}

pub fn download_multi_resume(_urls: &Vec<String>) -> Result<(), Box<dyn Error>> {
    Ok(())
}
