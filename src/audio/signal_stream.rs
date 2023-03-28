use std::io;

use color_eyre::Result;
use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::{CodecParameters, Decoder, DecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymphError,
    formats::FormatReader, io::{MediaSourceStream, ReadOnlySource},
};

pub struct SignalStream {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
}

impl SignalStream {
    pub fn new(format_reader: Box<dyn FormatReader>) -> Result<Self> {
        // Find the first audio track with a known (decodeable) codec.
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");

        // Use the default options for the decoder.
        let dec_opts: DecoderOptions = Default::default();

        // Create a decoder for the track.
        let decoder = symphonia::default::get_codecs().make(&track.codec_params, &dec_opts)?;

        let track_id = track.id;

        Ok(Self {
            format_reader,
            decoder,
            track_id,
        })
    }

    pub fn codec_params(&self) -> Option<&CodecParameters> {
        self.format_reader
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)
            .map(|t| &t.codec_params)
    }

    pub fn from_reader(reader: impl io::Read + Send + Sync + 'static) -> Result<Self> {
        let source = Box::new(ReadOnlySource::new(reader));
        // Open the media source.
        // Create the media source stream.
        let mss = MediaSourceStream::new(source, Default::default());

        // Use the default options for metadata and format readers.
        let hint = symphonia::core::probe::Hint::new();
        let meta_opts: symphonia::core::meta::MetadataOptions = Default::default();
        let fmt_opts: symphonia::core::formats::FormatOptions = Default::default();

        // Probe the media source.
        let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;

        // Get the instantiated format reader.
        let format = probed.format;

        Self::new(format)
    }
}

impl IntoIterator for SignalStream {
    type Item = (f32, f32);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(mut self) -> Self::IntoIter {
        Box::new(
            std::iter::from_fn(move || {
                loop {
                    let packet = match self.format_reader.next_packet() {
                        Ok(packet) => packet,
                        Err(SymphError::ResetRequired) => {
                            // The track list has been changed. Re-examine it and create a new set of decoders,
                            // then restart the decode loop. This is an advanced feature and it is not
                            // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                            // for chained OGG physical streams.
                            return None;
                        }
                        Err(err) => {
                            if let SymphError::IoError(err) = err {
                                if err.kind() == std::io::ErrorKind::UnexpectedEof {
                                    // The end of the stream has been reached.
                                    return None;
                                } else {
                                    dbg!(&err);
                                    // A unrecoverable error occured, halt decoding.
                                    panic!("{}", err);
                                }
                            } else {
                                dbg!(&err);
                                // A unrecoverable error occured, halt decoding.
                                panic!("{}", err);
                            }
                        }
                    };

                    // Consume any new metadata that has been read since the last packet.
                    while !self.format_reader.metadata().is_latest() {
                        // Pop the old head of the metadata queue.
                        self.format_reader.metadata().pop();

                        // Consume the new metadata at the head of the metadata queue.
                    }

                    // If the packet does not belong to the selected track, skip over it.
                    if packet.track_id() != self.track_id {
                        continue;
                    }

                    // Decode the packet into audio samples.
                    match self.decoder.decode(&packet) {
                        Ok(decoded) => {
                            match decoded {
                                AudioBufferRef::F32(buf) => {
                                    let chan_0 = buf.chan(0).to_owned();
                                    let chan_1 = buf.chan(1).to_owned();
                                    break Some(chan_0.into_iter().zip(chan_1.into_iter()));
                                }
                                _ => {
                                    // Repeat for the different sample formats.
                                    unimplemented!()
                                }
                            }
                        }
                        Err(SymphError::IoError(_)) => {
                            // The packet failed to decode due to an IO error, skip the packet.
                            continue;
                        }
                        Err(SymphError::DecodeError(_)) => {
                            // The packet failed to decode due to invalid data, skip the packet.
                            continue;
                        }
                        Err(err) => {
                            // An unrecoverable error occured, halt decoding.
                            panic!("{}", err);
                        }
                    }
                }
            })
            .flatten(),
        )
    }
}
