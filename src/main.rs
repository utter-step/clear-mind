use std::{fs, time::Duration};

use color_eyre::Result;
use humantime::format_duration;
use url::Url;

mod audio;
mod rss;

fn main() -> Result<()> {
    let path = "./mag-143.mp3";
    #[allow(unused_variables)]
    let stream = fs::File::open(path)?;

    let episode_url = "https://sphinx.acast.com/themagnusarchives/mag143-heartofdarkness/media.mp3";
    let stream = ureq::get(episode_url).call()?.into_reader();

    let gaps = audio::analyzer::find_gaps(stream)?;

    for gap in gaps {
        let timestamp = Duration::from_secs_f64(gap.start as f64 / gap.sample_rate as f64);
        println!("Silence at {}", format_duration(timestamp));
    }

    let podcast_url: Url = "https://rss.acast.com/themagnusarchives".parse()?;
    let episodes = rss::fetch_episodes(&podcast_url)?;

    dbg!(episodes);

    Ok(())
}
