use std::{fs, io, path::PathBuf};

use bevy::{
    app::AppExit,
    prelude::*,
    window::{AppLifecycle, WindowCloseRequested},
};

use crate::game::{Campaign, Game};

#[derive(Resource)]
pub struct SaveFile {
    path: PathBuf,
    last_saved: f32,
    revision: u64,
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
        }
    }

    pub fn load(&self) -> Option<Campaign> {
        let data = fs::read(&self.path).ok()?;
        match serde_json::from_slice(&data) {
            Ok(save) => Some(save),
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

    pub fn write(&self, campaign: &Campaign) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?
        }
        let temporary = self.path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_vec(campaign)?)?;
        fs::rename(temporary, &self.path)
    }
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
    if suspended || closed || file.revision != play.revision || now - file.last_saved >= 3. {
        match file.write(&game.campaign) {
            Ok(()) => {
                file.last_saved = now;
                file.revision = play.revision;
            }
            Err(error) => warn!("Spielstand konnte nicht gesichert werden: {error}"),
        }
    }
}
