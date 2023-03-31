use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub(crate) subcommand: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Command {
    /// Parse RSS feed
    ParseRss {
        /// RSS feed URL
        #[clap(short, long)]
        url: Url,
    },
    /// Find gaps in audio file or stream
    Podcast(PodcastSource),
}

#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct PodcastSource {
    /// File path to audio file
    pub file: Option<PathBuf>,
    /// URL to audio stream
    pub url: Option<Url>,
}
