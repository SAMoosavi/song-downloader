use clap::Parser;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use itertools::Itertools;
use rayon::iter::*;
use std::{collections::HashMap, fs, io::Write, path::PathBuf, sync::Arc};

fn download_song(tab: &Arc<Tab>) -> Result<String, Box<dyn std::error::Error>> {
    let urls: Vec<_> = tab
        .wait_for_elements("div.dl > div.link_dl > a.button--wayra")?
        .iter()
        .filter_map(|el| el.get_attribute_value("href").ok().flatten())
        .filter(|href| href.ends_with("mp3"))
        .collect();

    let best_url = match urls.len() {
        0 => format!("No mp3 URLs found: {}", tab.get_url()),
        1 => urls.first().ok_or("No URLs found")?.to_string(),
        _ => urls
            .iter()
            .find(|s| !s.contains("128"))
            .ok_or("No suitable URL found")?
            .to_string(),
    };

    Ok(best_url)
}

fn navigate_to_song(
    browser: &Browser,
    href: &str,
    exist_songs: &[String],
    artist_name: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let song_name = href
        .split('/')
        .rev()
        .nth(1)
        .ok_or("Invalid URL structure: unable to extract song name")?
        .replace(['-', '_'], " ")
        .replace(artist_name, "")
        .trim()
        .to_string();

    if exist_songs.contains(&song_name) {
        return Ok((song_name, String::new()));
    }

    let tab = browser.new_tab()?;
    tab.navigate_to(href)?;

    let result = download_song(&tab);
    tab.close_target()?;

    match result {
        Ok(url) => Ok((song_name, url)),
        Err(e) => Err(e),
    }
}

fn get_songs_name(
    artist_name: &str,
    path: PathBuf,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(path)?;
    let mut songs = Vec::new();

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".mp3") {
                    let song_name = file_name
                        .to_lowercase()
                        .replace(".mp3", "")
                        .replace(['-', '_'], " ")
                        .replace(artist_name, "")
                        .trim()
                        .to_string();
                    songs.push(song_name);
                }
            }
        } else if file_type.is_dir() {
            let mut sub_songs = get_songs_name(artist_name, entry.path())?;
            songs.append(&mut sub_songs);
        }
    }

    Ok(songs)
}

fn get_list_of_songs(artist_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let artists = fs::read_dir("/media/moosavi/files/music")?;

    let songs = artists
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let artist = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.replace(['-', '_'], " ").to_lowercase())?;

            if artist == artist_name {
                get_songs_name(artist_name, path).ok()
            } else {
                None
            }
        })
        .flatten()
        .collect();

    Ok(songs)
}

fn get_single_songs_urls(
    browser: &Browser,
    url: &str,
    artist_name: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let tab = browser.new_tab()?;

    let songs = get_list_of_songs(artist_name)?;

    tab.navigate_to(&format!("{url}/?section=music"))?;

    let elements =
        tab.wait_for_elements("section.artist > div.row > div.col-sm-3 > a:nth-child(1)")?;

    let urls = elements
        .par_iter()
        .filter_map(|song| song.get_attribute_value("href").ok().flatten())
        .filter_map(move |href| {
            for _ in 0..3 {
                if let Ok(url) = navigate_to_song(browser, &href, &songs, artist_name) {
                    return Some(url);
                }
            }
            println!("Failed to navigate to: {href}");
            None
        })
        .collect::<HashMap<_, _>>();
    tab.close_target()?;

    Ok(urls)
}

// ************************************************

fn download_album(tab: &Arc<Tab>) -> Result<String, Box<dyn std::error::Error>> {
    let elements = tab
        .wait_for_elements("a.button--wayra")
        .or_else(|_| tab.wait_for_elements(".details > p > a:nth-child(1)"))?;

    let urls: Vec<_> = elements
        .iter()
        .filter_map(|el| el.get_attribute_value("href").ok().flatten())
        .filter(|href| href.ends_with("zip"))
        .collect();

    let url = match urls.len() {
        0 => format!("No ZIP URLs found: {}", tab.get_url()),
        1 => urls.first().ok_or("No URLs found")?.to_string(),
        _ => urls
            .iter()
            .find(|s| !s.contains("128"))
            .ok_or("No suitable URL found (non-128kbps)")?
            .to_string(),
    };

    Ok(url)
}

fn navigate_to_album(
    browser: &Browser,
    href: &str,
    exist_albums: &[String],
    artist_name: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let album_name = href
        .split('/')
        .rev()
        .nth(1)
        .ok_or("Invalid URL structure: unable to extract song name")?
        .replace(['-', '_'], " ")
        .replace(artist_name, "")
        .trim()
        .to_string();

    if exist_albums.contains(&album_name) {
        return Ok((album_name, String::new()));
    }

    let tab = browser.new_tab()?;
    tab.navigate_to(href)?;

    let result = download_album(&tab);
    tab.close_target()?;

    match result {
        Ok(url) => Ok((album_name, url)),
        Err(e) => Err(e),
    }
}

fn get_albums_name(
    artist_name: &str,
    path: PathBuf,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(path)?;
    let mut albums = Vec::new();

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".mp3") {
                    let song_name = file_name
                        .to_lowercase()
                        .replace(".mp3", "")
                        .replace(['-', '_'], " ")
                        .replace(artist_name, "")
                        .trim()
                        .to_string();
                    albums.push(song_name);
                }
            }
        } else if file_type.is_dir() {
            let mut sub_albums = get_albums_name(artist_name, entry.path())?;
            albums.append(&mut sub_albums);
        }
    }

    Ok(albums)
}

fn get_list_of_albums(artist_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let artists = fs::read_dir("/media/moosavi/files/music")?;

    let albums = artists
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let artist = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.replace(['-', '_'], " ").to_lowercase())?;

            if artist == artist_name {
                get_albums_name(artist_name, path).ok()
            } else {
                None
            }
        })
        .flatten()
        .collect();

    Ok(albums)
}

fn get_album_urls(
    browser: &Browser,
    url: &str,
    artist_name: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let tab = browser.new_tab()?;

    let albums = get_list_of_albums(artist_name)?;

    tab.navigate_to(&format!("{url}/?section=album"))?;

    let elements =
        tab.wait_for_elements("section.artist > div.row > div.col-sm-3 > a:nth-child(1)")?;

    let urls = elements
        .par_iter()
        .filter_map(|album| album.get_attribute_value("href").ok().flatten())
        .filter_map(move |href| {
            for _ in 0..3 {
                if let Ok(url) = navigate_to_album(browser, &href, &albums, artist_name) {
                    return Some(url);
                }
            }
            println!("Failed to navigate to:: {href}");
            None
        })
        .collect::<HashMap<_, _>>();
    tab.close_target()?;

    Ok(urls)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Conf {
    #[arg(short, long)]
    artist_name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let artist_name = Conf::parse().artist_name;
    let url = format!("https://mymusicbaran1.ir/artists/{artist_name}");
    let artist_name = artist_name.replace(['-', '_'], " ").to_lowercase();

    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(false) // Set to false to show the browser
            .build()?,
    )?;

    let songs_url = get_single_songs_urls(&browser, &url, &artist_name)?;
    let albums_url = get_album_urls(&browser, &url, &artist_name)?;

    let mut file = fs::File::create(format!("{artist_name}.json"))?;
    file.write_all(format!("{:?}\n{:?}", songs_url, albums_url).as_bytes())?;

    Ok(())
}
