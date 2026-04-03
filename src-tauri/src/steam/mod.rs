use std::process::Command;
use std::path::PathBuf;

pub struct SteamCMD {
    path: PathBuf,
}

impl SteamCMD {
    pub fn new() -> Result<Self, String> {
        // Look for steamcmd in PATH, or common install locations
        let candidates = [
            "steamcmd",
            "/usr/games/steamcmd",
            "/usr/bin/steamcmd",
        ];

        for candidate in &candidates {
            if Command::new(candidate).arg("+quit").output().is_ok() {
                return Ok(SteamCMD { path: PathBuf::from(candidate) });
            }
        }

        Err("steamcmd not found. Please install it (e.g. nix-shell -p steamcmd)".into())
    }

    pub fn download_workshop_item(&self, app_id: u32, workshop_id: u64) -> Result<(), String> {
        let status = Command::new(&self.path)
            .args([
                "+login", "anonymous",
                "+workshop_download_item", &app_id.to_string(), &workshop_id.to_string(),
                "+quit",
            ])
            .status()
            .map_err(|e| format!("Failed to run SteamCMD: {}", e))?;

        if !status.success() {
            return Err("SteamCMD exited with error".to_string());
        }
        Ok(())
    }
}
