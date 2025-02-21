use rayon::iter::ParallelIterator;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use itertools::Itertools;
use rayon::iter::IntoParallelRefIterator;

fn download_song(tab: &Arc<Tab>) -> Result<String, Box<dyn std::error::Error>> {
    let urls: Vec<_> = tab
        .wait_for_elements("div.dl > div.link_dl > a.button--wayra")?
        .iter()
        .map(|el| el.get_attribute_value("href").unwrap().unwrap())
        .filter(|href| href.ends_with("mp3"))
        .collect();
    if urls.len() == 1 {
        Ok(urls[0].to_string())
    } else {
        let url = urls
            .iter()
            .filter(|s| s.find("128").is_none())
            .next()
            .unwrap();
        Ok(url.to_string())
    }
}

fn navigate_to_song(
    browser: &Browser,
    href: &str,
    exist_songs: &Vec<String>,
    artist_name: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let song_name = href
        .split('/')
        .rev()
        .nth(1)
        .unwrap()
        .replace("-", " ")
        .replace("_", " ")
        .replace(artist_name, "")
        .trim()
        .to_string();

    let url = if !exist_songs.contains(&song_name) {
        let tab = browser.new_tab()?;
        tab.navigate_to(&href)?;

        let url = download_song(&tab)?;
        tab.close_target()?;
        url
    } else {
        "".to_string()
    };

    Ok((song_name.replace("-", " "), url))
}

fn get_songs_name(
    artist_name: &str,
    path: PathBuf,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let entries: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();

    let enter_songs = entries
        .iter()
        .filter(|entry| entry.file_type().unwrap().is_file())
        .map(|entry| entry.file_name().to_str().unwrap().to_string())
        .filter(|name| name.ends_with(".mp3"))
        .map(|file_name| {
            file_name
                .to_lowercase()
                .replace(".mp3", "")
                .replace("-", " ")
                .replace("_", " ")
                .replace(artist_name, "")
                .trim()
                .to_string()
        });

    let mut songs: Vec<_> = entries
        .iter()
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| get_songs_name(artist_name, entry.path()))
        .flat_map(Result::ok)
        .concat();

    songs.extend(enter_songs);

    Ok(songs)
}

fn get_list_of_songs(artist_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let artists: Vec<_> = fs::read_dir("/media/moosavi/files/music")?
        .filter_map(Result::ok)
        .collect();

    let songs = artists
        .iter()
        .filter(|entry| {
            let path = entry.path();
            let artist = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace("-", " ")
                .replace("_", " ")
                .to_lowercase();

            artist == artist_name
        })
        .map(|entry| get_songs_name(&artist_name, entry.path()))
        .flat_map(Result::ok)
        .concat();

    Ok(songs)
}

fn get_single_songs_urls(
    browser: &Browser,
    url: &str,
    artist_name: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let tab = browser.new_tab()?;

    let songs = get_list_of_songs(&artist_name)?;

    tab.navigate_to(&format!("{url}/?section=music"))?;

    let elements =
        tab.wait_for_elements("section.artist > div.row > div.col-sm-3 > a:nth-child(1)")?;

    let urls = elements
        .par_iter()
        .map(|song| song.get_attribute_value("href").unwrap().unwrap())
        .map(|href| navigate_to_song(&browser, &href, &songs, &artist_name).unwrap())
        .collect::<HashMap<_, _>>();
    tab.close_target()?;

    Ok(urls)
}

// ************************************************

fn download_album2(tab: &Arc<Tab>) -> Result<String, Box<dyn std::error::Error>> {
    let urls: Vec<_> = tab
        .wait_for_elements(".details > p > a:nth-child(1)")?
        .iter()
        .map(|el| el.get_attribute_value("href").unwrap().unwrap())
        .filter(|href| href.ends_with("zip"))
        .collect();
    if urls.len() == 1 {
        Ok(urls[0].to_string())
    } else if urls.is_empty() {
        Ok("".to_string())
    } else {
        let url = urls
            .iter()
            .filter(|s| s.find("128").is_none())
            .next()
            .unwrap();
        Ok(url.to_string())
    }
}

fn download_album(tab: &Arc<Tab>) -> Result<String, Box<dyn std::error::Error>> {
    let urls: Vec<_> = tab
        .wait_for_elements("a.button--wayra")?
        .iter()
        .map(|el| el.get_attribute_value("href").unwrap().unwrap())
        .filter(|href| href.ends_with("zip"))
        .collect();
    if urls.len() == 1 {
        Ok(urls[0].to_string())
    } else if urls.is_empty() {
        Ok("".to_string())
    } else {
        let url = urls
            .iter()
            .filter(|s| s.find("128").is_none())
            .next()
            .unwrap();
        Ok(url.to_string())
    }
}

fn navigate_to_album(
    browser: &Browser,
    href: &str,
    exist_albums: &Vec<String>,
    artist_name: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let album_name = href
        .split('/')
        .rev()
        .nth(1)
        .unwrap()
        .replace("-", " ")
        .replace("_", " ")
        .replace(artist_name, "")
        .trim()
        .to_string();

    let url = if !exist_albums.contains(&album_name) {
        let tab = browser.new_tab()?;
        tab.navigate_to(&href)?;

        let url = match download_album(&tab) {
            Ok(x) => x,
            Err(_) => match download_album2(&tab) {
                Ok(x) => x,
                Err(e) => {
                    tab.close_target()?;

                    return Err(e);
                }
            },
        };
        tab.close_target()?;
        url
    } else {
        "".to_string()
    };

    Ok((album_name.replace("-", " "), url))
}

fn get_albums_name(
    artist_name: &str,
    path: PathBuf,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let entries: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();

    let enter_albums = entries
        .iter()
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().to_str().unwrap().to_string())
        .map(|file_name| {
            file_name
                .to_lowercase()
                .replace("-", " ")
                .replace("_", " ")
                .replace(artist_name, "")
                .trim()
                .to_string()
        });

    let mut albums: Vec<_> = entries
        .iter()
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| get_albums_name(artist_name, entry.path()))
        .filter_map(Result::ok)
        .concat();

    albums.extend(enter_albums);

    Ok(albums)
}

fn get_list_of_albums(artist_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let artists: Vec<_> = fs::read_dir("/media/moosavi/files/music")?
        .filter_map(Result::ok)
        .collect();

    let songs = artists
        .iter()
        .filter(|entry| {
            let path = entry.path();
            let artist = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace("-", " ")
                .replace("_", " ")
                .to_lowercase();

            artist == artist_name
        })
        .map(|entry| get_albums_name(&artist_name, entry.path()))
        .filter_map(Result::ok)
        .concat();

    Ok(songs)
}

fn get_album_urls(
    browser: &Browser,
    url: &str,
    artist_name: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let tab = browser.new_tab()?;

    let albums = get_list_of_albums(&artist_name)?;

    tab.navigate_to(&format!("{url}/?section=album"))?;

    let elements =
        tab.wait_for_elements("section.artist > div.row > div.col-sm-3 > a:nth-child(1)")?;

    let urls = elements
        .par_iter()
        .map(|album| album.get_attribute_value("href").unwrap().unwrap())
        .map(|href| navigate_to_album(&browser, &href, &albums, &artist_name).unwrap())
        .collect::<HashMap<_, _>>();
    tab.close_target()?;

    Ok(urls)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let artist_name = "moein";
    let url = format!("https://mymusicbaran1.ir/artists/{artist_name}");
    let artist_name = artist_name
        .replace("-", " ")
        .replace("_", " ")
        .to_lowercase();

    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(false) // Set to false to show the browser
            .build()
            .unwrap(),
    )?;

    let songs_url = get_single_songs_urls(&browser, &url, &artist_name)?;
    let albums_url = get_album_urls(&browser, &url, &artist_name)?;

    println!("{:?}", songs_url);
    println!("{:?}", albums_url);
    Ok(())
}
