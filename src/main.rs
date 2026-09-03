mod find_exist;

use clap::Parser;
use find_exist::{Exist, get_list_of_exist, normalize_name};
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use rayon::iter::*;
use std::{collections::HashMap, fs, io::Write, path::PathBuf, sync::Arc};
use serde::Serialize;

enum MediaType {
    Music,
    Album,
}

impl MediaType {
    fn file_extension(&self) -> &str {
        match self {
            MediaType::Music => ".mp3",
            MediaType::Album => ".zip",
        }
    }
}
impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Music => write!(f, "music"),
            MediaType::Album => write!(f, "album"),
        }
    }
}

fn get_urls(
    browser: &Browser,
    url: &str,
    artist_name: &str,
    exist: &Exist,
    page_type: MediaType,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let tab = browser.new_tab()?;

    tab.navigate_to(&format!("{}/?section={}", url, page_type))?;

    let elements =
        match tab.wait_for_elements("section.artist > div.row > div.col-sm-3 > a:nth-child(1)") {
            Ok(el) => el,
            Err(_) => {
                let _ = tab.close_target();
                return Ok(HashMap::new());
            }
        };

    let urls = elements
        .par_iter()
        .filter_map(|element| {
            let href = element.get_attribute_value("href").ok().flatten()?;

            let exist_list = match page_type {
                MediaType::Music => &exist.musics,
                MediaType::Album => &exist.albums,
            };

            match navigate_to_media(
                browser,
                &href,
                artist_name,
                exist_list,
                &page_type,
            ) {
                Ok((key, value)) if !value.is_empty() => Some((key, value)),
                Ok(_) => None,
                Err(e) => {
                    println!("Failed to process {}: {}", href, e);
                    None
                }
            }
        })
        .collect();

    let _ = tab.close_target();

    Ok(urls)
}

fn navigate_to_media(
    browser: &Browser,
    href: &str,
    artist_name: &str,
    exist: &[String],
    page_type: &MediaType,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let raw_name = href
        .split('/')
        .rev()
        .nth(1)
        .ok_or("Invalid URL structure: unable to extract music name")?;
    let name = normalize_name(&raw_name.replace(artist_name, ""));

    if exist.contains(&name) {
        return Ok((name, String::new()));
    }

    let tab = browser.new_tab()?;
    tab.navigate_to(href)?;

    let result = get_url(&tab, page_type);
    tab.close_target()?;

    match result {
        Ok(url) => Ok((name, url)),
        Err(e) => Err(e),
    }
}

fn get_url(tab: &Arc<Tab>, page_type: &MediaType) -> Result<String, Box<dyn std::error::Error>> {
    let elements = match page_type {
        MediaType::Music => tab.wait_for_elements("div.dl > div.link_dl > a.button--wayra")?,
        MediaType::Album => tab.wait_for_elements("a[href$='.zip']")?,
    };

    let urls: Vec<_> = elements
        .iter()
        .filter_map(|el| el.get_attribute_value("href").ok().flatten())
        .filter(|href| href.ends_with(page_type.file_extension()))
        .collect();

    let url = match urls.len() {
        0 => return Err(format!(
            "No {} URLs found: {}",
            page_type.file_extension().to_uppercase(),
            tab.get_url()
        ).into()),
        1 => urls.first().ok_or("No URLs found")?.to_string(),
        _ => urls
            .iter()
            .find(|s| !s.contains("128"))
            .ok_or("No suitable URL found (non-128kbps)")?
            .to_string(),
    };

    Ok(url)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Conf {
    #[arg(value_name = "ARTIST_NAME")]
    artist_name: String,

    #[arg(short, long, default_value = "~/Music")]
    music_dir: PathBuf,
}

#[derive(Serialize)]
struct Output {
    musics: HashMap<String, String>,
    albums: HashMap<String, String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = Conf::parse();
    let artist_name = conf.artist_name;
    let url = format!("https://musicbaran1.ir/artists/{}", urlencoding::encode(&artist_name));
    let artist_name = artist_name.replace(['-', '_'], " ").to_lowercase();

    let music_dir = if let Some(stripped) = conf.music_dir.to_str().and_then(|s| s.strip_prefix("~/")) {
        dirs::home_dir()
            .ok_or("Could not determine home directory")?
            .join(stripped)
    } else {
        conf.music_dir
    };

    let exist = get_list_of_exist(&artist_name, music_dir)?;

    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(false) // Set to false to show the browser
            .build()?,
    )?;

    let albums_url = get_urls(&browser, &url, &artist_name, &exist, MediaType::Album)?;
    let musics_url = get_urls(&browser, &url, &artist_name, &exist, MediaType::Music)?;

    let output = Output {
        musics: musics_url,
        albums: albums_url,
    };
    let mut file = fs::File::create(format!("{artist_name}.json"))?;
    file.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;

    Ok(())
}
