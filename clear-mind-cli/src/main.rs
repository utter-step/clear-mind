use std::{fs, time::Duration};

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use humantime::format_duration;

use clear_mind_core::{audio, gap::GapInfo, rss};

use cli::{Cli, Command, PodcastSource};

mod cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        Command::ParseRss { url } => {
            let feed = rss::fetch_episodes(&url)?;
            println!("{:#?}", feed);
        }
        Command::Podcast(source) => {
            let stream = match source {
                PodcastSource {
                    file: Some(path),
                    url: None,
                } => Box::new(fs::File::open(path)?),
                PodcastSource {
                    file: None,
                    url: Some(url),
                } => ureq::get(url.as_str()).call()?.into_reader(),
                _ => unreachable!("Invalid combination of arguments, clap should prevent this"),
            };

            let gaps = audio::analyzer::find_gaps(stream)?;
            let possible_boundary = GapInfo::find_boundary_gaps(&gaps)
                .ok_or_else(|| eyre!("No boundary gaps found"))?;

            println!(
                "Most likely the podcast starts at {start} and ends at {end}",
                start = format_duration(Duration::from_secs_f64(
                    possible_boundary.0.start as f64 / possible_boundary.0.sample_rate as f64
                )),
                end = format_duration(Duration::from_secs_f64(
                    possible_boundary.1.start as f64 / possible_boundary.1.sample_rate as f64
                )),
            );

            println!("Gaps:");
            for gap in gaps {
                let timestamp = Duration::from_secs_f64(gap.start as f64 / gap.sample_rate as f64);
                println!("Silence at {}", format_duration(timestamp));
            }
        }
    }

    Ok(())
}
