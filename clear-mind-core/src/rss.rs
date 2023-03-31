use std::io::{self, BufReader};

use color_eyre::{eyre::eyre, Result};
use url::Url;

#[derive(Debug, Clone)]
pub struct Episode {
    pub title: String,
    pub url: String,
}

/// Fetch podcast episodes from a RSS feed.
///
/// It is assumed that the episodes are attached as enclosures in RSS feed.
pub fn fetch_episodes(url: &Url) -> Result<Vec<Episode>> {
    let response = ureq::get(url.as_str()).call()?;

    fetch_episodes_from_stream(response.into_reader())
}

fn fetch_episodes_from_stream(
    reader: impl io::Read + Send + Sync + 'static,
) -> Result<Vec<Episode>> {
    let buf_reader = BufReader::new(reader);
    let channel = rss::Channel::read_from(buf_reader)?;

    let episodes = channel
        .items()
        .iter()
        .map(|item| {
            Ok(Episode {
                title: item
                    .title()
                    .ok_or_else(|| eyre!("{item:?} title is not set"))?
                    .to_owned(),
                url: item
                    .enclosure()
                    .ok_or_else(|| eyre!("{item:?} enclosure is not set"))?
                    .url()
                    .to_owned(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(episodes)
}
