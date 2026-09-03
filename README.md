# Song Downloader

A CLI tool to scrape **MusicBaran** and retrieve direct download links for songs/albums by a given artist.

## Features

- Scrapes **MusicBaran** for songs and albums by a specified artist.
- Extracts direct download URLs (prefers 320kbps over 128kbps).
- Skips already-downloaded songs/albums by scanning a local music directory.
- Ignore list to skip specific songs/albums via JSON file.
- Outputs a clean JSON file with `{ "musics": {}, "albums": {} }` structure.
- Runs headless by default, with `--headless` flag available.

## Installation

Requires **Rust** (edition 2024).

```sh
git clone https://github.com/SAMoosavi/song-downloader.git
cd song-downloader
cargo build --release
```

## Usage

```sh
./target/release/song-downloader "artist-name" --music-dir /path/to/music
```

### Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `ARTIST_NAME` | Artist name (supports `-` and `_` separators) | *required* |
| `--music-dir` | Local music directory to scan for existing files | `~/Music` |
| `--headless` | Run browser in headless mode | `false` |
| `--ignore` | Path to ignore list JSON file | `music_dir/ignore_download_song.json` |

### Examples

```sh
# Basic usage
./target/release/song-downloader "siavash ghomayshi"

# Custom music directory
./target/release/song-downloader "siavash ghomayshi" --music-dir "/run/media/sam/music/Siavash Ghomayshi/"

# Headless mode with custom ignore file
./target/release/song-downloader "siavash ghomayshi" --music-dir ~/Music/Artist/ --headless --ignore ~/my-ignore.json
```

## Ignore List

Create an `ignore_download_song.json` file in your music directory (or specify a custom path with `--ignore`):

```json
{
  "musics": ["khazoon 2", "mohabat remix"],
  "albums": ["singles", "yadegari 2"]
}
```

Names are matched case-insensitively with spaces normalized (hyphens/underscores become spaces).

## Output

The tool creates a JSON file named `<artist-name>.json` in the current directory:

```json
{
  "musics": {
    "taghdim": "https://dl.dlmusicbaran.ir/...",
    "nakhoda": "https://dl.dlmusicbaran.ir/..."
  },
  "albums": {
    "neghab": "https://dl.dlmusicbaran.ir/..."
  }
}
```

## License

Contributions & Issues welcome via pull requests.
