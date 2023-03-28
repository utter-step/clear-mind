use std::io::Read;

use color_eyre::{Result, eyre::{ensure, eyre}};
use simple_moving_average::{SingleSumSMA, SMA};
use static_assertions::{const_assert_eq};

use super::signal_stream::SignalStream;

/// Fixed sample rate we're able to analyze.
///
/// Currently, only 44.1kHz audio is supported
const SUPPORTED_SAMPLE_RATE: usize = 44100;
/// Window size in milliseconds
const WINDOW_SIZE_MS: usize = 500;
/// How much samples fits in a window
const WINDOW_SIZE_SAMPLES: usize = SUPPORTED_SAMPLE_RATE * WINDOW_SIZE_MS / 1000;

const_assert_eq!(SUPPORTED_SAMPLE_RATE * WINDOW_SIZE_MS % 1000, 0);

/// Audio signal cap, anything under it considered silence
const SIGNAL_POWER_CAP: f64 = 0.00015;

/// Info about gap
#[derive(Debug, Clone, Copy)]
pub struct GapInfo {
    /// Start of a gap (sample index)
    pub start: usize,
    /// Length of a gap (in samples)
    pub length: usize,
    /// Sample rate per second
    pub sample_rate: usize,
}

/// Analyzes an audio stream and returns a list of gaps
pub fn find_gaps(stream_reader: impl Read + Send + Sync + 'static) -> Result<Vec<GapInfo>> {
    let signal_stream = SignalStream::from_reader(stream_reader)?;
    let codec_params = signal_stream.codec_params().ok_or_else(|| eyre!("codec params are unavailable"))?;

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| eyre!("sample rate is not set"))?;

    ensure!(sample_rate == 44100, "currently only 44.1kHz audio is supported");

    let mean_signal = signal_stream
        .into_iter()
        .map(|(l, r)| ((l + r) / 2.).powi(2) as f64)
        .fuse();

    let mut mean_power_window = SingleSumSMA::<f64, f64, WINDOW_SIZE_SAMPLES>::new();
    // skip first second of an audio
    let mut to_skip = SUPPORTED_SAMPLE_RATE;

    let mut gaps = Vec::new();

    for (i, sample) in mean_signal.enumerate() {
        mean_power_window.add_sample(sample);

        if mean_power_window.get_average().sqrt() < SIGNAL_POWER_CAP && to_skip == 0 {
            let start_of_gap = i - mean_power_window.get_sample_window_size();
            let gap_length = mean_power_window.get_sample_window_size();

            gaps.push(GapInfo {
                start: start_of_gap,
                length: gap_length,
                sample_rate: SUPPORTED_SAMPLE_RATE,
            });

            to_skip = SUPPORTED_SAMPLE_RATE / 2;
        }

        to_skip = to_skip.saturating_sub(1);
    }

    Ok(gaps)
}
