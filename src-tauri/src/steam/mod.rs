use directories::ProjectDirs;
use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tar::Archive;

pub const WALLPAPER_ENGINE_APP_ID: u32 = 431960;
const STEAMCMD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz";

#[derive(Clone, Serialize)]
pub struct SteamState {
    pub installed: bool,
    pub binary_path: Option<String>,
    pub managed_install: bool,
    pub root_dir: String,
    pub message: String,
}

pub struct SteamCMD {
    path: PathBuf,
    root_dir: PathBuf,
}

impl SteamCMD {
    pub fn state() -> Result<SteamState, String> {
        let root_dir = managed_root_dir()?;

        match Self::new() {
            Ok(steam) => {
                let managed_install = steam.path.starts_with(&root_dir);
                Ok(SteamState {
                    installed: true,
                    binary_path: Some(steam.path.to_string_lossy().into_owned()),
                    managed_install,
                    root_dir: steam.root_dir.to_string_lossy().into_owned(),
                    message: if managed_install {
                        "SteamCMD is installed in the app-managed runtime.".into()
                    } else {
                        "SteamCMD is available from your system.".into()
                    },
                })
            }
            Err(_) => Ok(SteamState {
                installed: false,
                binary_path: None,
                managed_install: false,
                root_dir: root_dir.to_string_lossy().into_owned(),
                message:
                    "SteamCMD is missing. Install it from the app to enable workshop downloads."
                        .into(),
            }),
        }
    }

    pub fn new() -> Result<Self, String> {
        let root_dir = managed_root_dir()?;
        let local_binary = root_dir.join("steamcmd.sh");

        let candidates = [
            local_binary,
            PathBuf::from("steamcmd"),
            PathBuf::from("/usr/games/steamcmd"),
            PathBuf::from("/usr/bin/steamcmd"),
        ];

        for candidate in candidates {
            if command_exists(&candidate) {
                return Ok(Self {
                    path: candidate,
                    root_dir: root_dir.clone(),
                });
            }
        }

        Err("SteamCMD was not found.".into())
    }

    pub fn install_managed() -> Result<SteamState, String> {
        let root_dir = managed_root_dir()?;
        fs::create_dir_all(&root_dir).map_err(|e| e.to_string())?;

        let response = reqwest::blocking::get(STEAMCMD_URL)
            .map_err(|e| format!("Failed to download SteamCMD: {e}"))?;
        let response = response
            .error_for_status()
            .map_err(|e| format!("SteamCMD download failed: {e}"))?;
        let archive_bytes = response
            .bytes()
            .map_err(|e| format!("Failed to read SteamCMD archive: {e}"))?;

        let tar = GzDecoder::new(Cursor::new(archive_bytes));
        let mut archive = Archive::new(tar);
        archive
            .unpack(&root_dir)
            .map_err(|e| format!("Failed to unpack SteamCMD: {e}"))?;

        let binary = root_dir.join("steamcmd.sh");
        if !binary.exists() {
            return Err("SteamCMD download finished but steamcmd.sh was not found.".into());
        }

        Ok(SteamState {
            installed: true,
            binary_path: Some(binary.to_string_lossy().into_owned()),
            managed_install: true,
            root_dir: root_dir.to_string_lossy().into_owned(),
            message: "SteamCMD was installed into the app runtime.".into(),
        })
    }

    pub fn download_workshop_item(&self, workshop_id: u64) -> Result<PathBuf, String> {
        fs::create_dir_all(&self.root_dir).map_err(|e| e.to_string())?;

        run_command(
            Command::new(&self.path).args([
                "+@ShutdownOnFailedCommand",
                "1",
                "+@NoPromptForPassword",
                "1",
                "+force_install_dir",
                &self.root_dir.to_string_lossy(),
                "+login",
                "anonymous",
                "+workshop_download_item",
                &WALLPAPER_ENGINE_APP_ID.to_string(),
                &workshop_id.to_string(),
                "+quit",
            ]),
            "SteamCMD exited with an error while downloading the wallpaper",
        )?;

        let workshop_path = self.workshop_path(workshop_id);
        if !workshop_path.exists() {
            return Err(
                "SteamCMD completed, but the workshop item folder was not found afterwards.".into(),
            );
        }

        Ok(workshop_path)
    }

    pub fn workshop_path(&self, workshop_id: u64) -> PathBuf {
        self.root_dir
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join(WALLPAPER_ENGINE_APP_ID.to_string())
            .join(workshop_id.to_string())
    }
}

fn managed_root_dir() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from("online", "opencom", "wallpaper-engine-linux")
        .ok_or_else(|| "Failed to resolve the app data directory.".to_string())?;
    Ok(dirs.data_local_dir().join("steamcmd"))
}

fn command_exists(candidate: &Path) -> bool {
    let command = Command::new(candidate).arg("+quit").output();
    command.is_ok()
}

fn run_command(command: &mut Command, context: &str) -> Result<(), String> {
    let output = command.output().map_err(|e| format!("{context}: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    Err(format_command_failure(context, &output))
}

fn format_command_failure(context: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let details = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("process exited with {}", output.status)
    };

    format!("{context}: {details}")
}
