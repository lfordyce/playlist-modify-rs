# playlist-modify-rs

## Summary
Simple CLI to read in a m3u8 manifest and modify the ordering of renditions / variants.

## Usage
```shell
cargo run -- --help
```

```shell
Usage: playlist-modify-rs [OPTIONS]

Options:
  -i, --input <PLAYLIST SOURCE>  Sources for input
  -f, --field <FIELD>            Field to sort on for playlist variants [default: bandwidth] [possible values: bandwidth, avg-bandwidth, resolution]
  -d, --direction <DIRECTION>    Sort direction [default: asc] [possible values: asc, desc]
  -h, --help                     Print help
```
## Run Options
```shell
# Pipe in local playlist through stdin and apply sort options, resulting playlist will be printed to stdout.
cat assets/playlist.m3u8 | cargo run -- -f resolution -d desc
# Supply a remote URL to source master playlist and apply sort options, resulting playlist will be printed to stdout.
cargo run -- -i https://lw.bamgrid.com/2.0/hls/vod/bam/ms02/hls/dplus/bao/master_unenc_hdr10_all.m3u8 -f bandwidth -d asc
# Capture results into .m3u8 file
cat assets/playlist.m3u8 | cargo run > results.m3u8
# In release mode
cargo run --release -- --input=https://lw.bamgrid.com/2.0/hls/vod/bam/ms02/hls/dplus/bao/master_unenc_hdr10_all.m3u8
```