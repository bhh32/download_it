use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about, author, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Single {
        #[arg(
            short,
            long,
            default_missing_value = "true",
            num_args = 0..=1,
            help = "Resume a paused/stopped download."
        )]
        resume: Option<bool>,
        #[arg(
            short,
            long,
            num_args = 0..=1,
            help = "Use a cookie file."
        )]
        cookie: Option<String>,
        #[arg(
            short = 'H',
            long,
            num_args = 0..=20,
            help = "Enter header arguments for more complex downloads."
        )]
        header_args: Option<Vec<String>>,
        #[arg(
            short = 'p',
            long,
            num_args = 0..=1,
            help = "The directory the file will be downloaded to."
        )]
        file_path: Option<String>,
        #[arg(
            short = 'n',
            long,
            num_args = 0..=1,
            help = "The name for the file being downloaded."
        )]
        file_name: Option<String>,
        /// The download link.
        url: String,
    },
    Multi {
        #[arg(
            short,
            long,
            default_missing_value = "true",
            num_args = 0..=1,
            help = "Resume multiple paused/stopped downloads."
        )]
        resume: Option<bool>,
        #[arg(
            short,
            long,
            num_args = 0..1,
            help = "Use a cookie file for more complex downloads."
        )]
        cookie: Option<String>,
        #[arg(
            short = 'H',
            long,
            num_args = 0..=20,
            help = "Enter header arguments for more complex downloads."
        )]
        header_args: Option<Vec<String>>,
        #[arg(
            short = 'p',
            long,
            num_args = 0..=1,
            help = "The directory to download the files to."
        )]
        file_path: Option<String>,
        #[arg(
            short = 'n',
            long,
            num_args = 0..=1000,
            help = "The file names to save each file to. Note: Keep them in the same order as the URLs or they will be misnamed."
        )]
        file_names: Option<Vec<String>>,
        /// The list of download links separated by a space.
        urls: Vec<String>,
    },
}
