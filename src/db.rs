use crate::download::{Download, DownloadStatus};
use indicatif::ProgressBar;
use rusqlite::Connection;
use std::{
    error::Error,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    sync::Arc,
};

#[derive(Debug)]
pub struct ResumeDb {
    conn: Connection,
}

impl ResumeDb {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let db_path = dirs::config_dir()
            .expect("Config directory")
            .as_path()
            .join("download_it");

        if !db_path.exists() {
            match std::fs::create_dir_all(&db_path) {
                Ok(_) => println!("Database path created successfully!"),
                Err(e) => {
                    eprintln!("Resume database path failed to be created: {e}");
                    return Err(Box::new(IoError::new(
                        IoErrorKind::Other,
                        "Could not create the resume database path in config directory.",
                    )));
                }
            };
        }

        let db_path = db_path.join("resume.db");

        let conn = match Connection::open(db_path) {
            Ok(conn) => conn,
            Err(e) => return Err(Box::new(e)),
        };

        // Create the table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS resumes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                status TEXT NOT NULL,
                error TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn create_resume(&self, download: &Download) -> Result<(), Box<dyn Error>> {
        let status = serde_json::to_string(&download.status)?;
        let err = if let Some(err) = download.error.clone() {
            err
        } else {
            String::new()
        };
        self.conn.execute(
            "INSERT INTO resumes (url, file_name, file_path, status, error)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &download.url,
                &download.file_name,
                &download.file_path,
                &status,
                &err,
            ],
        )?;

        Ok(())
    }

    pub fn get_resume(&self, url: &str) -> Result<Option<Download>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT url, file_name, file_path, status, error FROM resumes WHERE url = ?1",
        )?;

        let mut rows = stmt.query_map([url], |row| {
            let status_json: String = row.get(3)?;
            let status: DownloadStatus = serde_json::from_str(&status_json).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?;

            let error_str: String = row.get(4)?;
            let error = if error_str.is_empty() {
                None
            } else {
                Some(error_str)
            };

            let progress_bar = Arc::new(ProgressBar::new(0));

            Ok(Download {
                url: row.get(0)?,
                file_name: row.get(1)?,
                file_path: row.get(2)?,
                progress_bar,
                status,
                error,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn delete_resume(&self, download: &Download) -> Result<(), Box<dyn Error>> {
        self.conn
            .execute("DELETE FROM resumes WHERE url = ?1", [&download.url])?;

        Ok(())
    }

    pub fn update_resume(&self, download: &Download) -> Result<(), Box<dyn Error>> {
        let status = serde_json::to_string(&download.status)?;
        let err = if let Some(err) = download.error.clone() {
            err
        } else {
            String::new()
        };

        self.conn.execute(
            "UPDATE resumes
             SET file_name = ?1, file_path = ?2, status = ?3, error = ?4, updated_at = CURRENT_TIMESTAMP
             WHERE url = ?5",
             [
                 &download.file_name,
                 &download.file_path,
                 &status,
                 &err,
                 &download.url,
             ]
        )?;

        Ok(())
    }
}
