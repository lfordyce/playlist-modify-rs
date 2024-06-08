use std::cmp::Ordering;
use std::io;
use std::io::BufReader;

use clap::Parser;
use either::Either;
use reqwest::blocking::Client;
use tap::Pipe;

pub use cli::Direction;

use crate::cli::{Config, InputType, VariantSortFieldName};
use crate::read::{Entry, MasterPlaylist};
use crate::sort::SortableByField;

mod cli;

pub mod attribute;
mod read;
mod sort;

struct Input {
    rx: std::sync::mpsc::Receiver<MasterPlaylist>,
}

impl From<InputType> for Input {
    fn from(value: InputType) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || -> anyhow::Result<()> {
            let buf_read = match value {
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
            .pipe(BufReader::new);
            let playlist = read::parse_playlist(buf_read);

            tx.send(playlist)?;

            Ok(())
        });

        Self { rx }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let mut config = Config::parse();

    let input = Input::from(config.input.take().unwrap_or_default());
    while let Ok(mut parsed) = input.rx.recv() {
        let fields = vec![sort::SortField::<VariantSortFieldName> {
            field: config.field,
            direction: config.direction,
        }];
        sort::by_fields(&mut parsed.entries, &fields);
        parsed.write_to(&mut io::stdout()).expect("write to stdout");
    }
    Ok(())
}

impl SortableByField<VariantSortFieldName> for Entry {
    fn sort(&self, rhs: &Self, field: &VariantSortFieldName) -> Ordering {
        match field {
            VariantSortFieldName::Bandwidth => Ord::cmp(
                &self.get_bandwidth().unwrap_or(0),
                &rhs.get_bandwidth().unwrap_or(0),
            ),
            VariantSortFieldName::AvgBandwidth => Ord::cmp(
                &self.get_average_bandwidth().unwrap_or(0),
                &rhs.get_average_bandwidth().unwrap_or(0),
            ),
            VariantSortFieldName::Resolution => Ord::cmp(
                &self.get_resolution_height().unwrap_or(0),
                &rhs.get_resolution_height().unwrap_or(0),
            ),
        }
    }
}
