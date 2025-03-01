mod find_exist;

use clap::Parser;
use find_exist::{get_list_of_exist, Exist};
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use rayon::iter::*;
use std::{collections::HashMap, fs, io::Write, path::PathBuf, sync::Arc};

trait MediaCollection {
    type CollectionType;
    fn get<'a>(&'a self, t: &MediaType) -> &'a Self::CollectionType;
}

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

impl MediaCollection for Exist {
    type CollectionType = Vec<String>;
    fn get<'a>(&'a self, t: &MediaType) -> &'a Self::CollectionType {
        match t {
            MediaType::Music => &self.musics,
            MediaType::Album => &self.albums,
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

            match navigate_to_media(
                browser,
                &href,
                artist_name,
                exist.get(&page_type),
                &page_type,
            ) {
                Ok((key, value)) => Some((key, value)),
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
    let name = href
        .split('/')
        .rev()
        .nth(1)
        .ok_or("Invalid URL structure: unable to extract music name")?
        .replace(['-', '_'], " ")
        .replace(artist_name, "")
        .trim()
        .to_string();

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
        MediaType::Album => tab
            .wait_for_elements("a.button--wayra")
            .or_else(|_| tab.wait_for_elements(".details > p > a:nth-child(1)"))?,
    };

    let urls: Vec<_> = elements
        .iter()
        .filter_map(|el| el.get_attribute_value("href").ok().flatten())
        .filter(|href| href.ends_with(page_type.file_extension()))
        .collect();

    let url = match urls.len() {
        0 => format!(
            "No {} URLs found: {}",
            page_type.file_extension().to_uppercase(),
            tab.get_url()
        ),
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

    #[arg(short, long, default_value = "/media/moosavi/files/music")]
    music_dir: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = Conf::parse();
    let artist_name = conf.artist_name;
    let url = format!("https://mymusicbaran1.ir/artists/{artist_name}");
    let artist_name = artist_name.replace(['-', '_'], " ").to_lowercase();
    let exist = get_list_of_exist(&artist_name, conf.music_dir)?;

    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(true) // Set to false to show the browser
            .build()?,
    )?;

    let albums_url = get_urls(&browser, &url, &artist_name, &exist, MediaType::Album)?;
    let musics_url = get_urls(&browser, &url, &artist_name, &exist, MediaType::Music)?;

    let mut file = fs::File::create(format!("{artist_name}.json"))?;
    file.write_all(format!("{:?}\n{:?}", musics_url, albums_url).as_bytes())?;

    Ok(())
}
