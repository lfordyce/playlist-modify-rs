use clap::{Parser, ValueEnum};

#[derive(Parser, Clone, Debug)]
pub struct Config {
    /// Sources for input.
    #[arg(short, long, value_name = "PLAYLIST SOURCE")]
    pub input: Option<InputType>,

    /// Field to sort on for playlist variants
    #[arg(short, long, value_enum, default_value_t = VariantSortFieldName::Bandwidth)]
    pub field: VariantSortFieldName,

    /// Sort direction
    #[arg(short, long, value_enum, default_value_t = Direction::Asc)]
    pub direction: Direction,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ValueEnum)]
pub enum Direction {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ValueEnum)]
pub enum VariantSortFieldName {
    #[default]
    Bandwidth,
    AvgBandwidth,
    Resolution,
}

#[derive(Debug, Default, Clone)]
pub enum InputType {
    #[default]
    Stdin,
    Url(reqwest::Url),
}

impl From<&str> for InputType {
    fn from(s: &str) -> Self {
        InputType::Url(reqwest::Url::parse(s).expect("valid URL"))
    }
}
