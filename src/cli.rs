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
            require_equals = true,
            default_missing_value = "true",
            required = false,
            num_args = 0..=1,
            help = "Resume a paused/stopped download."
        )]
        resume: Option<bool>,
        /// The download link.
        file: String,
    },
    Multi {
        #[arg(
            short,
            long,
            require_equals = true,
            default_missing_value = "true",
            required = false,
            num_args = 0..=1,
            help = "Resume multiple paused/stopped downloads."
        )]
        resume: Option<bool>,
        /// The list of download links separated by a space.
        files: Vec<String>,
    },
}
