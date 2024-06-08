use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};

use crate::attribute::AttributePairs;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct MasterPlaylist {
    pub valid_header: bool,
    pub independent_segments: bool,
    pub entries: Vec<Entry>,
}

impl MasterPlaylist {
    pub(crate) const HEADER: &'static str = "#EXTM3U";
    pub(crate) const INDEPENDENT_SEGMENTS: &'static str = "#EXT-X-INDEPENDENT-SEGMENTS";

    pub fn write_to<T: Write>(&self, w: &mut T) -> std::io::Result<()> {
        write!(w, "{}", self)?;
        Ok(())
    }
}

impl Display for MasterPlaylist {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#EXTM3U")?;
        if self.independent_segments {
            writeln!(f, "#EXT-X-INDEPENDENT-SEGMENTS")?;
        }
        for ent in &self.entries {
            writeln!(f, "{}", ent)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attributes(pub(crate) BTreeMap<String, String>);

impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut attrs = self.0.iter().peekable();
        while let Some((k, v)) = attrs.next() {
            if attrs.peek().is_none() {
                write!(f, "{}={}", k, v)?;
            } else {
                write!(f, "{}={},", k, v)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    ExtXStreamInf { attributes: Attributes, uri: String },
    ExtXIFrame { attributes: Attributes },
    Alternative { attributes: Attributes },
}

impl Entry {
    pub(crate) const PREFIX: &'static str = "#EXT-X-MEDIA:";
    pub(crate) const PREFIX_EXTXIFRAME: &'static str = "#EXT-X-I-FRAME-STREAM-INF:";
    pub(crate) const PREFIX_EXTXSTREAMINF: &'static str = "#EXT-X-STREAM-INF:";

    pub fn get_bandwidth(&self) -> Option<u64> {
        match self {
            Entry::ExtXStreamInf { attributes, .. } => attributes
                .0
                .get("BANDWIDTH")
                .map(|b| b.parse::<u64>().unwrap()),
            Entry::ExtXIFrame { attributes, .. } => attributes
                .0
                .get("BANDWIDTH")
                .map(|b| b.parse::<u64>().unwrap()),
            _ => None,
        }
    }

    pub fn get_average_bandwidth(&self) -> Option<u64> {
        match self {
            Entry::ExtXStreamInf { attributes, .. } => attributes
                .0
                .get("AVERAGE-BANDWIDTH")
                .map(|ab| ab.parse::<u64>().unwrap()),
            Entry::ExtXIFrame { attributes, .. } => attributes
                .0
                .get("AVERAGE-BANDWIDTH")
                .map(|ab| ab.parse::<u64>().unwrap()),
            _ => None,
        }
    }

    pub fn get_resolution_height(&self) -> Option<u64> {
        match self {
            Entry::ExtXStreamInf { attributes, .. } => attributes.0.get("RESOLUTION").map(|r| {
                r.split_once('x')
                    .map(|(_width, height)| height.parse::<u64>().unwrap_or(0))
                    .unwrap_or(0)
            }),
            Entry::ExtXIFrame { attributes, .. } => attributes.0.get("RESOLUTION").map(|r| {
                r.split_once('x')
                    .map(|(_width, height)| height.parse::<u64>().unwrap_or(0))
                    .unwrap_or(0)
            }),
            _ => None,
        }
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Entry::ExtXStreamInf { attributes, uri } => {
                write!(f, "{}{}", Self::PREFIX_EXTXSTREAMINF, attributes)?;
                write!(f, "\n{}", uri)?;
            }
            Entry::ExtXIFrame { attributes } => {
                write!(f, "{}{}", Self::PREFIX_EXTXIFRAME, attributes)?;
            }
            Entry::Alternative { attributes } => {
                write!(f, "{}{}", Self::PREFIX, attributes)?;
            }
        }
        Ok(())
    }
}

fn tag<T>(input: &str, tag: T) -> &str
where
    T: AsRef<str>,
{
    input.trim().split_at(tag.as_ref().len()).1
}

fn collect_btree<'a>(
    items: impl IntoIterator<Item = (&'a str, &'a str)> + 'a,
) -> BTreeMap<String, String> {
    items
        .into_iter()
        .map(|(key, val)| (key.to_owned(), val.to_owned()))
        .collect::<BTreeMap<String, String>>()
}

pub fn parse_playlist<T: Read>(reader: BufReader<T>) -> MasterPlaylist {
    let mut lines = reader
        .lines()
        .map_while(Result::ok)
        .filter(|v| !v.is_empty());

    let mut entires = Vec::new();
    let mut independent_segments = false;
    let mut header = false;

    while let Some(l) = lines.next() {
        match l.as_str() {
            s if s.starts_with(Entry::PREFIX_EXTXSTREAMINF) => {
                if let Some(uri) = lines.next() {
                    let remainder = tag(s, Entry::PREFIX_EXTXSTREAMINF);

                    let ent = Entry::ExtXStreamInf {
                        attributes: Attributes(collect_btree(AttributePairs::new(remainder))),
                        uri,
                    };
                    entires.push(ent);
                } else {
                    continue;
                }
            }
            s if s.starts_with(Entry::PREFIX_EXTXIFRAME) => {
                let remainder = tag(s, Entry::PREFIX_EXTXIFRAME);
                let ent = Entry::ExtXIFrame {
                    attributes: Attributes(collect_btree(AttributePairs::new(remainder))),
                };
                entires.push(ent);
            }
            s if s.starts_with(Entry::PREFIX) => {
                let remainder = tag(s, Entry::PREFIX);
                let ent = Entry::Alternative {
                    attributes: Attributes(collect_btree(AttributePairs::new(remainder))),
                };
                entires.push(ent);
            }
            s if s.starts_with(MasterPlaylist::INDEPENDENT_SEGMENTS) => {
                independent_segments = true;
            }
            s if s.starts_with(MasterPlaylist::HEADER) => {
                header = true;
            }
            _ => {
                println!("UNKNOWN")
            }
        }
    }

    MasterPlaylist {
        valid_header: header,
        independent_segments,
        entries: entires,
    }
}

#[cfg(test)]
mod tests {
    use crate::read::parse_playlist;
    use std::io::BufReader;

    #[test]
    fn basic_parse_alt() {
        let f = std::fs::File::open("assets/playlist.m3u8").unwrap();

        let playlist = parse_playlist(BufReader::new(f));
        println!("{}", playlist)
    }
}
