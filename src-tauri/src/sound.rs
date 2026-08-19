//! Sound library: merged view of built-in (resource dir) and user-imported
//! (app data dir) audio files. Ids are file stems; user files shadow built-ins
//! with the same stem.

use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tauri::AppHandle;

use crate::settings;

const AUDIO_EXTS: [&str; 4] = ["wav", "mp3", "ogg", "flac"];

#[derive(Clone, Serialize)]
pub struct SoundInfo {
    pub id: String,
    pub name: String,
    pub builtin: bool,
    #[serde(skip)]
    pub path: PathBuf,
}

fn scan_dir(dir: &PathBuf, builtin: bool, out: &mut Vec<SoundInfo>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        let is_audio = path
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| AUDIO_EXTS.contains(&x.to_ascii_lowercase().as_str()))
            .unwrap_or(false);
        if !is_audio {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        out.push(SoundInfo {
            id: stem.to_string(),
            name: stem.to_string(),
            builtin,
            path,
        });
    }
}

/// Merged list: user imports first (shadowing built-ins by id), then built-ins,
/// each group sorted by name for a stable dropdown.
pub fn scan(app: &AppHandle) -> Vec<SoundInfo> {
    let mut user = Vec::new();
    scan_dir(&settings::user_sound_dir(app), false, &mut user);
    let mut builtin = Vec::new();
    scan_dir(&settings::builtin_sound_dir(app), true, &mut builtin);

    let mut by_id: std::collections::HashMap<String, SoundInfo> = Default::default();
    for s in builtin {
        by_id.entry(s.id.clone()).or_insert(s);
    }
    for s in user {
        by_id.insert(s.id.clone(), s); // user shadows built-in
    }
    let mut all: Vec<_> = by_id.into_values().collect();
    all.sort_by(|a, b| a.name.cmp(&b.name));
    all
}
