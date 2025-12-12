# 🎵 Song Downloader

A CLI tool to scrape **MusicBaran** and retrieve direct download links for songs by a given artist.

## 🚀 Features
- Scrapes **MusicBaran** for songs by a specified artist.
- Extracts direct URLs of music files.
- Check doesn't exist music (default: `~/Music`).

## 📦 Installation
Ensure you have **Rust** installed, then build the project:

```sh
git clone https://github.com/SAMoosavi/song-downloader.git
cd song-downloader
cargo build --release
```

## 🛠️ Usage

Run the program with an artist's name:

```sh
./target/release/song-downloader "Artist Name"
```

By default, songs are saved in `~/Music`. You can specify a custom directory:

```sh
./target/release/song-downloader "Artist Name" --music-dir "/path/to/music"
```

## 📜 Example
```sh
./target/release/song-downloader "siavash ghomayshi"
```
🔹 This will fetch download links for **siavash-ghomayshi**'s songs soen't exist them in `~/Music`.

⭐ **Contributions & Issues**  
Feel free to submit **issues** or **pull requests** to improve the tool!  
```

This `README.md` provides a clean, structured introduction with clear usage instructions. Let me know if you'd like any modifications! 🚀
