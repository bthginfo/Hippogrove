use std::{fs, io, path::PathBuf};

use bevy::{
    app::AppExit,
    prelude::*,
    window::{AppLifecycle, WindowCloseRequested},
};

use serde::Deserialize;

use crate::{
    game::{Campaign, Game, Saves},
    sound::Mix,
};

#[derive(Deserialize)]
#[serde(untagged)]
enum SavedFile {
    Profiles(Saves),
    Legacy(Campaign),
}

#[derive(Resource)]
pub struct SaveFile {
    path: PathBuf,
    last_saved: f32,
    revision: u64,
    mix: Mix,
    reading: crate::display::ReadingSize,
}

impl SaveFile {
    pub fn for_device() -> Self {
        #[cfg(target_os = "android")]
        let root = bevy::window::ANDROID_APP
            .get()
            .and_then(|app| app.internal_data_path())
            .expect("Android supplies the private app data directory");
        #[cfg(target_os = "ios")]
        let root = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library/Application Support/Hippogrove");
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let root = std::env::var_os("HIPPOGROVE_SAVE_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("hippogrove"))
            })
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|path| PathBuf::from(path).join(".local/share/hippogrove"))
            })
            .unwrap_or_else(|| PathBuf::from("build/hippogrove-save"));
        Self {
            path: root.join("campaign.json"),
            last_saved: 0.,
            revision: 0,
            mix: Mix::default(),
            reading: Default::default(),
        }
    }

    pub fn load_reading(&mut self) -> crate::display::ReadingSize {
        self.reading = fs::read(self.path.with_file_name("anzeige.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        self.reading
    }

    pub fn load_mix(&mut self) -> Mix {
        self.mix = fs::read(self.path.with_file_name("ton.json"))
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .unwrap_or_default();
        self.mix
    }

    pub fn load(&self) -> Option<Saves> {
        let data = fs::read(&self.path).ok()?;
        match serde_json::from_slice(&data) {
            Ok(SavedFile::Profiles(saved)) => Some(saved),
            Ok(SavedFile::Legacy(campaign)) => Some(Saves::legacy(campaign)),
            Err(error) => {
                warn!("Spielstand konnte nicht gelesen werden: {error}");
                None
            }
        }
    }

    pub fn preserve_unreadable(&self) {
        if self.path.is_file() {
            let backup = self.path.with_extension("previous.json");
            if !backup.exists() {
                if let Err(error) = fs::copy(&self.path, &backup) {
                    warn!("Sicherung des alten Spielstands fehlgeschlagen: {error}");
                }
            }
        }
    }

    pub fn write(&self, saves: &Saves) -> io::Result<()> {
        write_json(&self.path, saves)
    }
}

fn write_json(path: &std::path::Path, value: &impl serde::Serialize) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, serde_json::to_vec(value)?)?;
    fs::rename(temporary, path)
}

pub fn checkpoint(
    time: Res<Time>,
    game: Res<Game>,
    play: Res<crate::play::Play>,
    mut file: ResMut<SaveFile>,
    mut lifecycle: EventReader<AppLifecycle>,
    mut closing: EventReader<WindowCloseRequested>,
    mut exits: EventReader<AppExit>,
) {
    let suspended = lifecycle
        .read()
        .any(|state| matches!(state, AppLifecycle::WillSuspend | AppLifecycle::Suspended));
    let closed = closing.read().count() > 0 || exits.read().count() > 0;
    let now = time.elapsed_secs();
    if file.reading != play.reading {
        let path = file.path.with_file_name("anzeige.json");
        match write_json(&path, &play.reading) {
            Ok(()) => file.reading = play.reading,
            Err(error) => warn!("Anzeige konnte nicht gesichert werden: {error}"),
        }
    }
    if file.mix != play.mix {
        let path = file.path.with_file_name("ton.json");
        match write_json(&path, &play.mix) {
            Ok(()) => file.mix = play.mix,
            Err(error) => warn!("Ton-Einstellungen konnten nicht gesichert werden: {error}"),
        }
    }
    if suspended || closed || file.revision != play.revision || now - file.last_saved >= 3. {
        match file.write(&game.saved_games()) {
            Ok(()) => {
                file.last_saved = now;
                file.revision = play.revision;
            }
            Err(error) => warn!("Spielstand konnte nicht gesichert werden: {error}"),
        }
    }
}
