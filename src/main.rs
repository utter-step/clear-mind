use std::{time::Duration, fs};

use color_eyre::Result;
use humantime::format_duration;

mod audio;

fn main() -> Result<()> {
    let path = "./mag-143.mp3";
    #[allow(unused_variables)]
    let stream = fs::File::open(path)?;

    let url = "https://sphinx.acast.com/themagnusarchives/mag143-heartofdarkness/media.mp3";
    let stream = ureq::get(url).call()?.into_reader();

    let gaps = audio::analyzer::find_gaps(stream)?;

    for gap in gaps {
        let timestamp = Duration::from_secs_f64(gap.start as f64 / gap.sample_rate as f64);
        println!("Silence at {}", format_duration(timestamp));
    }

    Ok(())
}
