//! Clipboard Core: the one representation of a clip and the one list that
//! holds them.
//!
//! Every other family (search, navigation, previews, actions, management)
//! reads from this list instead of keeping its own copy of clipboard data.

use serde::{Deserialize, Serialize};

use crate::settings::Settings;
use crate::storage::Paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipKind {
    Text,
    Link,
    Image,
    Files,
}

/// The application a clip came from (app-awareness). Shared by the source
/// indicator in the UI and by the excluded-applications rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceApp {
    /// Display name, e.g. `Visual Studio Code`.
    pub name: String,
    /// Lowercase executable file name, e.g. `code.exe`. Identity key for
    /// exclusions and icon caching.
    pub exe_name: String,
    /// Full path of the executable, when it could be resolved.
    #[serde(default)]
    pub exe_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMeta {
    pub width: u32,
    pub height: u32,
    pub byte_len: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRef {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub kind: ClipKind,
    /// Unix milliseconds of the most recent time this content was copied.
    pub created_at: i64,
    #[serde(default)]
    pub pinned: bool,
    /// Plain text payload for `Text` and `Link` clips.
    #[serde(default)]
    pub text: Option<String>,
    /// Rich (HTML) payload when the source provided one. Kept so a normal
    /// paste can restore formatting and "paste as plain text" can drop it.
    #[serde(default)]
    pub html: Option<String>,
    /// First URL found in `text`, for `Link` clips.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub image: Option<ImageMeta>,
    #[serde(default)]
    pub files: Vec<FileRef>,
    #[serde(default)]
    pub source: Option<SourceApp>,
    /// Content fingerprint used to collapse repeated copies of the same thing.
    pub fingerprint: String,
}

impl Clip {
    /// Short, single-paragraph text used by previews and search. Never the
    /// full payload: the UI must stay fast with large clips.
    pub fn preview_text(&self) -> String {
        const MAX: usize = 400;

        let raw = match self.kind {
            ClipKind::Text | ClipKind::Link => self.text.clone().unwrap_or_default(),
            ClipKind::Image => self
                .image
                .map(|meta| format!("Image {}x{}", meta.width, meta.height))
                .unwrap_or_else(|| "Image".to_string()),
            ClipKind::Files => self
                .files
                .iter()
                .map(|file| file.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        };

        let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if collapsed.chars().count() > MAX {
            collapsed.chars().take(MAX).collect()
        } else {
            collapsed
        }
    }

    /// Text the UI searches against. Includes the source app so "copied from
    /// Chrome" style queries work without a second index.
    pub fn search_text(&self) -> String {
        let mut haystack = self.preview_text().to_lowercase();

        if let Some(url) = &self.url {
            haystack.push(' ');
            haystack.push_str(&url.to_lowercase());
        }

        for file in &self.files {
            haystack.push(' ');
            haystack.push_str(&file.path.to_lowercase());
        }

        if let Some(source) = &self.source {
            haystack.push(' ');
            haystack.push_str(&source.name.to_lowercase());
        }

        haystack
    }
}

/// Shape sent to the frontend. Binary payloads are never inlined: images are
/// fetched lazily per visible item through `get_clip_thumbnail`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipView {
    pub id: String,
    pub kind: ClipKind,
    pub created_at: i64,
    pub pinned: bool,
    pub preview: String,
    pub search_text: String,
    pub char_count: usize,
    pub url: Option<String>,
    pub has_html: bool,
    pub image: Option<ImageMeta>,
    pub files: Vec<FileRef>,
    pub source: Option<SourceApp>,
}

impl From<&Clip> for ClipView {
    fn from(clip: &Clip) -> Self {
        Self {
            id: clip.id.clone(),
            kind: clip.kind,
            created_at: clip.created_at,
            pinned: clip.pinned,
            preview: clip.preview_text(),
            search_text: clip.search_text(),
            char_count: clip.text.as_ref().map(|text| text.chars().count()).unwrap_or(0),
            url: clip.url.clone(),
            has_html: clip.html.is_some(),
            image: clip.image,
            files: clip.files.clone(),
            source: clip.source.clone(),
        }
    }
}

/// Ordered clipboard history. Newest first; pinned clips are exempt from the
/// history limit but keep their position in time.
#[derive(Debug, Default)]
pub struct History {
    items: Vec<Clip>,
    dirty: bool,
}

impl History {
    pub fn load(paths: &Paths) -> Self {
        let items = match std::fs::read(&paths.history) {
            Ok(bytes) => serde_json::from_slice::<Vec<Clip>>(&bytes).unwrap_or_else(|err| {
                log::warn!("history file unreadable, starting empty: {err}");
                Vec::new()
            }),
            Err(_) => Vec::new(),
        };

        let mut history = Self {
            items,
            dirty: false,
        };
        history.drop_missing_assets(paths);
        history
    }

    /// Images live on disk; a clip whose file vanished would render as a broken
    /// preview, so it is pruned at startup.
    fn drop_missing_assets(&mut self, paths: &Paths) {
        let before = self.items.len();
        self.items.retain(|clip| match clip.kind {
            ClipKind::Image => paths.image(&clip.id).exists(),
            _ => true,
        });

        if self.items.len() != before {
            self.dirty = true;
        }
    }

    pub fn items(&self) -> &[Clip] {
        &self.items
    }

    pub fn views(&self) -> Vec<ClipView> {
        self.items.iter().map(ClipView::from).collect()
    }

    pub fn get(&self, id: &str) -> Option<&Clip> {
        self.items.iter().find(|clip| clip.id == id)
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::replace(&mut self.dirty, false)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Inserts a newly captured clip. Re-copying existing content moves that
    /// clip back to the top instead of creating a duplicate.
    ///
    /// Returns ids whose on-disk assets must be removed (history-limit
    /// eviction), which the caller performs outside the lock.
    pub fn insert(&mut self, clip: Clip, settings: &Settings) -> Vec<String> {
        self.dirty = true;

        if let Some(index) = self
            .items
            .iter()
            .position(|item| item.fingerprint == clip.fingerprint)
        {
            let mut existing = self.items.remove(index);
            existing.created_at = clip.created_at;
            existing.source = clip.source.clone().or(existing.source);
            self.items.insert(0, existing);

            // The incoming clip is redundant; discard any asset it wrote.
            return vec![clip.id];
        }

        self.items.insert(0, clip);
        self.trim(settings.history_limit)
    }

    /// Enforces the configured history limit, skipping pinned clips.
    fn trim(&mut self, limit: usize) -> Vec<String> {
        let mut evicted = Vec::new();
        let mut unpinned = self.items.iter().filter(|clip| !clip.pinned).count();

        if unpinned <= limit {
            return evicted;
        }

        // Remove oldest unpinned clips first.
        for index in (0..self.items.len()).rev() {
            if unpinned <= limit {
                break;
            }
            if self.items[index].pinned {
                continue;
            }
            evicted.push(self.items.remove(index).id);
            unpinned -= 1;
        }

        evicted
    }

    /// Re-applies the limit after the user lowered it in settings.
    pub fn apply_limit(&mut self, limit: usize) -> Vec<String> {
        let evicted = self.trim(limit);
        if !evicted.is_empty() {
            self.dirty = true;
        }
        evicted
    }

    pub fn toggle_pin(&mut self, id: &str) -> Option<bool> {
        let clip = self.items.iter_mut().find(|clip| clip.id == id)?;
        clip.pinned = !clip.pinned;
        self.dirty = true;
        Some(clip.pinned)
    }

    pub fn remove(&mut self, id: &str) -> Option<String> {
        let index = self.items.iter().position(|clip| clip.id == id)?;
        let clip = self.items.remove(index);
        self.dirty = true;
        Some(clip.id)
    }

    /// Clears history. Pinned clips are removed too: this is an explicit
    /// user action on the whole list.
    pub fn clear(&mut self) -> Vec<String> {
        self.dirty = true;
        self.items.drain(..).map(|clip| clip.id).collect()
    }

    pub fn save(&self, paths: &Paths) -> std::io::Result<()> {
        let bytes = serde_json::to_vec(&self.items)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        crate::storage::write_atomic(&paths.history, &bytes)
    }

    /// Applications observed in history, used to offer exclusion candidates in
    /// settings instead of enumerating the whole system.
    pub fn known_apps(&self) -> Vec<SourceApp> {
        let mut apps: Vec<SourceApp> = Vec::new();

        for clip in &self.items {
            if let Some(source) = &clip.source {
                if !apps.iter().any(|app| app.exe_name == source.exe_name) {
                    apps.push(source.clone());
                }
            }
        }

        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        apps
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_clip(id: &str, text: &str) -> Clip {
        Clip {
            id: id.to_string(),
            kind: ClipKind::Text,
            created_at: now_ms(),
            pinned: false,
            text: Some(text.to_string()),
            html: None,
            url: None,
            image: None,
            files: Vec::new(),
            source: None,
            fingerprint: format!("text:{text}"),
        }
    }

    #[test]
    fn recopying_moves_clip_to_top_without_duplicating() {
        let settings = Settings::default();
        let mut history = History::default();

        history.insert(text_clip("a", "one"), &settings);
        history.insert(text_clip("b", "two"), &settings);
        let discarded = history.insert(text_clip("c", "one"), &settings);

        assert_eq!(history.items().len(), 2);
        assert_eq!(history.items()[0].id, "a");
        assert_eq!(discarded, vec!["c".to_string()]);
    }

    #[test]
    fn limit_evicts_oldest_but_keeps_pinned() {
        let settings = Settings {
            history_limit: 2,
            ..Settings::default()
        };
        let mut history = History::default();

        history.insert(text_clip("a", "one"), &settings);
        history.toggle_pin("a");
        history.insert(text_clip("b", "two"), &settings);
        history.insert(text_clip("c", "three"), &settings);
        let evicted = history.insert(text_clip("d", "four"), &settings);

        assert_eq!(evicted, vec!["b".to_string()]);
        assert!(history.get("a").is_some());
        assert_eq!(history.items().len(), 3);
    }
}
