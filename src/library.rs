use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Local,
    Stream,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaItem {
    pub title: String,
    pub uri: Url,
    pub kind: MediaKind,
}

impl MediaItem {
    pub fn source_label(&self) -> String {
        match self.kind {
            MediaKind::Local => self
                .uri
                .to_file_path()
                .ok()
                .map_or_else(|| self.uri.to_string(), |path| path.display().to_string()),
            MediaKind::Stream => self.uri.host_str().unwrap_or("network stream").to_owned(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("source does not exist: {0}")]
    Missing(PathBuf),
    #[error("unsupported source: {0}")]
    Unsupported(String),
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to fetch playlist {url}: {source}")]
    Fetch { url: String, source: reqwest::Error },
    #[error("playlist request failed for {url}: HTTP {status}")]
    HttpStatus {
        url: String,
        status: reqwest::StatusCode,
    },
    #[error("cannot convert path to a file URI: {0}")]
    InvalidPath(PathBuf),
}

pub fn load_sources(sources: &[String]) -> Result<Vec<MediaItem>, LibraryError> {
    let mut items = Vec::new();
    load_sources_incremental(sources, |item| items.push(item))?;
    Ok(items)
}

pub fn load_sources_incremental<F>(sources: &[String], mut add: F) -> Result<(), LibraryError>
where
    F: FnMut(MediaItem),
{
    let mut seen = HashSet::new();
    for source in sources {
        load_source(source, &mut |item| {
            if seen.insert(item.uri.clone()) {
                add(item);
            }
        })?;
    }
    Ok(())
}

fn load_source<F>(source: &str, add: &mut F) -> Result<(), LibraryError>
where
    F: FnMut(MediaItem),
{
    if let Ok(url) = Url::parse(source) {
        if matches!(url.scheme(), "http" | "https" | "file") {
            if matches!(url.scheme(), "http" | "https") && is_playlist_url(&url) {
                parse_remote_playlist(url, add)?;
                return Ok(());
            }
            add(item_from_url(url, None)?);
            return Ok(());
        }
        return Err(LibraryError::Unsupported(source.to_owned()));
    }

    let expanded = expand_home(source);
    let path = expanded.as_path();
    if path.is_dir() {
        scan_directory(path, add)?;
    } else if is_playlist(path) {
        parse_playlist(path, add)?;
    } else if path.is_file() && is_audio(path) {
        add(item_from_path(path, None)?);
    } else if !path.exists() {
        return Err(LibraryError::Missing(path.to_owned()));
    } else {
        return Err(LibraryError::Unsupported(source.to_owned()));
    }
    Ok(())
}

fn scan_directory<F>(directory: &Path, add: &mut F) -> Result<(), LibraryError>
where
    F: FnMut(MediaItem),
{
    let mut entries = fs::read_dir(directory)
        .map_err(|source| LibraryError::Read {
            path: directory.to_owned(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| LibraryError::Read {
            path: directory.to_owned(),
            source,
        })?;
    entries.sort_by_key(std::fs::DirEntry::path);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(&path, add)?;
        } else if is_audio(&path) {
            add(item_from_path(&path, None)?);
        }
    }
    Ok(())
}

fn parse_playlist<F>(path: &Path, add: &mut F) -> Result<(), LibraryError>
where
    F: FnMut(MediaItem),
{
    let content = fs::read_to_string(path).map_err(|source| LibraryError::Read {
        path: path.to_owned(),
        source,
    })?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    parse_playlist_content(&content, add, |line| {
        let expanded = expand_home(line);
        let entry = expanded.as_path();
        if entry.is_absolute() {
            Ok(Url::from_file_path(entry)
                .map_err(|()| LibraryError::InvalidPath(entry.to_owned()))?)
        } else {
            let resolved = base.join(entry);
            Ok(Url::from_file_path(resolved)
                .map_err(|()| LibraryError::InvalidPath(base.join(entry)))?)
        }
    })
}

fn parse_remote_playlist<F>(url: Url, add: &mut F) -> Result<(), LibraryError>
where
    F: FnMut(MediaItem),
{
    let response = reqwest::blocking::get(url.as_str()).map_err(|source| LibraryError::Fetch {
        url: url.to_string(),
        source,
    })?;
    let response = response.error_for_status().map_err(|source| {
        if let Some(status) = source.status() {
            LibraryError::HttpStatus {
                url: url.to_string(),
                status,
            }
        } else {
            LibraryError::Fetch {
                url: url.to_string(),
                source,
            }
        }
    })?;
    let content = response.text().map_err(|source| LibraryError::Fetch {
        url: url.to_string(),
        source,
    })?;
    parse_playlist_content(&content, add, |line| {
        url.join(line)
            .map_err(|source| LibraryError::Unsupported(source.to_string()))
    })
}

fn parse_playlist_content<F>(
    content: &str,
    add: &mut impl FnMut(MediaItem),
    resolve_entry: F,
) -> Result<(), LibraryError>
where
    F: Fn(&str) -> Result<Url, LibraryError>,
{
    let mut next_title = None;
    for raw_line in content.lines() {
        let line = raw_line.trim().trim_start_matches('\u{feff}');
        if let Some(metadata) = line.strip_prefix("#EXTINF:") {
            next_title = metadata
                .split_once(',')
                .map(|(_, title)| title.trim().to_owned())
                .filter(|title| !title.is_empty());
        } else if line.is_empty() || line.starts_with('#') {
            continue;
        } else {
            let url = if let Ok(url) = Url::parse(line) {
                url
            } else {
                resolve_entry(line)?
            };
            if !matches!(url.scheme(), "http" | "https" | "file") {
                return Err(LibraryError::Unsupported(line.to_owned()));
            }
            add(item_from_url(url, next_title.take())?);
        }
    }
    Ok(())
}

fn item_from_path(path: &Path, title: Option<String>) -> Result<MediaItem, LibraryError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|source| LibraryError::Read {
                path: path.to_owned(),
                source,
            })?
            .join(path)
    };
    let fallback = absolute
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .to_owned();
    let uri = Url::from_file_path(&absolute).map_err(|()| LibraryError::InvalidPath(absolute))?;
    Ok(MediaItem {
        title: title.unwrap_or(fallback),
        uri,
        kind: MediaKind::Local,
    })
}

fn item_from_url(url: Url, title: Option<String>) -> Result<MediaItem, LibraryError> {
    if url.scheme() == "file" {
        let path = url
            .to_file_path()
            .map_err(|()| LibraryError::Unsupported(url.to_string()))?;
        return item_from_path(&path, title);
    }
    let fallback = url
        .path_segments()
        .and_then(Iterator::last)
        .filter(|value| !value.is_empty())
        .or_else(|| url.host_str())
        .unwrap_or("Internet stream")
        .to_owned();
    Ok(MediaItem {
        title: title.unwrap_or(fallback),
        uri: url,
        kind: MediaKind::Stream,
    })
}

fn is_playlist(path: &Path) -> bool {
    extension(path).is_some_and(|value| matches!(value.as_str(), "m3u" | "m3u8"))
}

fn is_playlist_url(url: &Url) -> bool {
    url.path_segments()
        .and_then(Iterator::last)
        .is_some_and(|segment| {
            let path = Path::new(segment);
            extension(path).is_some_and(|value| matches!(value.as_str(), "m3u" | "m3u8"))
        })
}

fn is_audio(path: &Path) -> bool {
    extension(path).is_some_and(|value| {
        matches!(
            value.as_str(),
            "aac" | "flac" | "m4a" | "mp3" | "oga" | "ogg" | "opus" | "wav" | "webm"
        )
    })
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
}

fn expand_home(value: &str) -> PathBuf {
    if value == "~" {
        return BaseDirs::new().map_or_else(|| PathBuf::from(value), |dirs| dirs.home_dir().into());
    }
    if let Some(rest) = value.strip_prefix("~/")
        && let Some(dirs) = BaseDirs::new()
    {
        return dirs.home_dir().join(rest);
    }
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_audio_files_recursively_in_sorted_order() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("album")).unwrap();
        fs::write(directory.path().join("b.ogg"), []).unwrap();
        fs::write(directory.path().join("album/a.mp3"), []).unwrap();
        fs::write(directory.path().join("ignore.txt"), []).unwrap();
        let items = load_sources(&[directory.path().display().to_string()]).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "a");
        assert_eq!(items[1].title, "b");
    }

    #[test]
    fn parses_extinf_urls_and_relative_paths() {
        let directory = tempfile::tempdir().unwrap();
        let playlist = directory.path().join("radio.m3u8");
        fs::write(
            &playlist,
            "#EXTM3U\n#EXTINF:-1,Example Radio\nhttps://radio.example/live\n#EXTINF:42,Local Song\nmusic/song.flac\n",
        )
        .unwrap();
        let items = load_sources(&[playlist.display().to_string()]).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Example Radio");
        assert_eq!(items[0].kind, MediaKind::Stream);
        assert_eq!(items[1].title, "Local Song");
        assert_eq!(items[1].kind, MediaKind::Local);
        assert!(items[1].uri.as_str().ends_with("/music/song.flac"));
    }

    #[test]
    fn removes_duplicate_locations() {
        let sources = [
            "https://radio.example/live".to_owned(),
            "https://radio.example/live".to_owned(),
        ];
        assert_eq!(load_sources(&sources).unwrap().len(), 1);
    }

    #[test]
    fn incremental_loading_emits_items_in_source_order() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("first.mp3"), []).unwrap();
        fs::write(directory.path().join("second.ogg"), []).unwrap();
        let mut titles = Vec::new();
        load_sources_incremental(&[directory.path().display().to_string()], |item| {
            titles.push(item.title);
        })
        .unwrap();
        assert_eq!(titles, ["first", "second"]);
    }
}
