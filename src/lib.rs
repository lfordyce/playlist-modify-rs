use std::cmp::Ordering;
use std::io;
use std::io::Read;

use clap::Parser;
use either::Either;
use m3u8_rs::Playlist;
use reqwest::blocking::Client;
use tap::Pipe;

pub use cli::Direction;

use crate::cli::{Config, InputType, VariantSortFieldName};
use crate::sort::SortableByField;

mod cli;

mod sort;

struct Input {
    rx: std::sync::mpsc::Receiver<Playlist>,
}

impl From<InputType> for Input {
    fn from(value: InputType) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || -> anyhow::Result<()> {
            let playlist = match value {
                InputType::Stdin => Either::Left(io::stdin()),
                InputType::Url(url) => {
                    let client = Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .danger_accept_invalid_certs(true)
                        .build()
                        .expect("Http client build failed");
                    let resp = client.get(url).send()?;
                    Either::Right(resp)
                }
            }
            .pipe(|mut rdr| {
                let mut bytes: Vec<u8> = Vec::new();
                rdr.read_to_end(&mut bytes)
                    .expect("read all bytes from source input");
                m3u8_rs::parse_playlist_res(&bytes).expect("valid m3u8 playlist")
            });

            tx.send(playlist)?;

            Ok(())
        });

        Self { rx }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let mut config = Config::parse();

    let input = Input::from(config.input.take().unwrap_or_default());
    while let Ok(parsed) = input.rx.recv() {
        match parsed {
            Playlist::MasterPlaylist(mut pl) => {
                let fields = vec![sort::SortField::<VariantSortFieldName> {
                    field: config.field,
                    direction: config.direction,
                }];
                sort::by_fields(&mut pl.variants, &fields);

                pl.write_to(&mut io::stdout())
                    .expect("write modified m3u8 to stdout");
            }
            Playlist::MediaPlaylist(pl) => println!("Media playlist:\n{:?}", pl),
        }
    }
    Ok(())
}

impl SortableByField<VariantSortFieldName> for m3u8_rs::VariantStream {
    fn sort(&self, rhs: &Self, field: &VariantSortFieldName) -> Ordering {
        match field {
            VariantSortFieldName::Bandwidth => Ord::cmp(&self.bandwidth, &rhs.bandwidth),
            VariantSortFieldName::AvgBandwidth => Ord::cmp(
                &self.average_bandwidth.unwrap_or(0),
                &rhs.average_bandwidth.unwrap_or(0),
            ),
            VariantSortFieldName::Resolution => Ord::cmp(
                &self.resolution.map(|r| r.height).unwrap_or(0),
                &rhs.resolution.map(|r| r.height).unwrap_or(0),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use m3u8_rs::Resolution;

    use super::*;

    #[test]
    fn sort_variant_bandwidth() {
        let mut renditions = vec![
            m3u8_rs::VariantStream {
                is_i_frame: false,
                uri: "".to_string(),
                bandwidth: 4686817,
                average_bandwidth: None,
                codecs: None,
                resolution: Some(Resolution {
                    width: 1280,
                    height: 720,
                }),
                frame_rate: None,
                hdcp_level: None,
                audio: None,
                video: None,
                subtitles: None,
                closed_captions: None,
                other_attributes: None,
            },
            m3u8_rs::VariantStream {
                is_i_frame: false,
                uri: "".to_string(),
                bandwidth: 21551664,
                average_bandwidth: None,
                codecs: None,
                resolution: Some(Resolution {
                    width: 3840,
                    height: 2160,
                }),
                frame_rate: None,
                hdcp_level: None,
                audio: None,
                video: None,
                subtitles: None,
                closed_captions: None,
                other_attributes: None,
            },
            m3u8_rs::VariantStream {
                is_i_frame: false,
                uri: "".to_string(),
                bandwidth: 4686819,
                average_bandwidth: None,
                codecs: None,
                resolution: Some(Resolution {
                    width: 1280,
                    height: 720,
                }),
                frame_rate: None,
                hdcp_level: None,
                audio: None,
                video: None,
                subtitles: None,
                closed_captions: None,
                other_attributes: None,
            },
        ];

        let fields = vec![sort::SortField::<VariantSortFieldName> {
            field: VariantSortFieldName::Bandwidth,
            direction: Direction::Desc,
        }];
        sort::by_fields(&mut renditions, &fields);

        for (i, bandwidth) in [21551664, 4686819, 4686817].iter().enumerate() {
            assert_eq!(&renditions[i].bandwidth, bandwidth);
        }
    }
}
