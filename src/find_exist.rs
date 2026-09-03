use std::{error::Error, fs, path::PathBuf};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Exist {
    pub albums: Vec<String>,
    pub musics: Vec<String>,
}

pub fn get_list_of_exist(artist_name: &str, music_dir: PathBuf) -> Result<Exist, Box<dyn Error>> {
    // If music_dir itself is the artist dir, use it directly; otherwise search inside it
    let artist_dirs = if normalize_name(
        music_dir.file_name().and_then(|n| n.to_str()).unwrap_or(""),
    ) == normalize_name(artist_name)
    {
        vec![music_dir]
    } else {
        find_artist_directories(music_dir, artist_name)?
    };

    // Process all found artist directories
    let mut albums = Vec::new();
    let mut musics = Vec::new();

    for artist_dir in &artist_dirs {
        let (albums_in_dir, musics_in_dir) = process_artist_directory(artist_dir.clone(), artist_name)?;
        albums.extend(albums_in_dir);
        musics.extend(musics_in_dir);
    }

    Ok(Exist { albums, musics })
}

// Helper function to find artist directories
fn find_artist_directories(
    root: PathBuf,
    target_name: &str,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let normalized_target = normalize_name(target_name);

    let artist_dirs = root
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let dir_name = path.file_name()?.to_str()?;
            let normalized = normalize_name(dir_name);
            (normalized == normalized_target).then_some(path)
        })
        .collect::<Vec<_>>();

    Ok(artist_dirs)
}

// Process a single artist directory
fn process_artist_directory(
    artist_dir: PathBuf,
    artist_name: &str,
) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    let mut albums = Vec::new();
    let mut musics = Vec::new();

    for entry in artist_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            // Process album directory
            if let Some(album_name) = path.file_name().and_then(|n| n.to_str()) {
                albums.push(album_name.to_lowercase());

                // Process musics in album
                musics.extend(process_album_directory(path, artist_name)?);
            }
        }
    }

    Ok((albums, musics))
}

// Process musics in an album directory
fn process_album_directory(
    album_dir: PathBuf,
    artist_name: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut musics = Vec::new();

    for entry in album_dir.read_dir()? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && let Some(music_name) = process_music_entry(&entry, artist_name)
        {
            musics.push(music_name.to_lowercase());
        }
    }

    Ok(musics)
}

// Process individual music entry
fn process_music_entry(entry: &fs::DirEntry, artist_name: &str) -> Option<String> {
    let binding = entry.file_name();
    let file_name = binding.to_str()?;

    if !file_name.ends_with(".mp3") {
        return None;
    }

    Some(normalize_music_name(file_name, artist_name))
}

// Normalize music name
fn normalize_music_name(file_name: &str, artist_name: &str) -> String {
    let step1 = file_name
        .trim_end_matches(".mp3")
        .replace(['-', '_'], " ");

    match step1.to_lowercase().find("  ") {
        Some(pos) => step1[pos + 2..].to_lowercase().trim().to_string(),
        None => step1.to_lowercase().replace(artist_name, "").trim().to_string(),
    }
}

// Normalize directory names for comparison
pub fn normalize_name(name: &str) -> String {
    name.replace(['-', '_'], " ")
        .to_lowercase()
        .trim()
        .to_string()
}
