use std::process::Command;
use std::path::PathBuf;

pub struct SteamCMD {
    path: PathBuf,
}

impl SteamCMD {
    pub fn new() -> Result<Self, String> {
        Ok(SteamCMD { path: PathBuf::from("steamcmd") })
    }

    pub fn download_workshop_item(&self, app_id: u32, workshop_id: u64) -> Result<(), String> {
        let status = Command::new(&self.path)
            .args([
                "+login", "anonymous",
                "+workshop_download_item", &app_id.to_string(), &workshop_id.to_string(),
                "+quit"
            ])
            .status()
            .map_err(|e| format!("Failed to run SteamCMD: {}", e))?;

        if !status.success() {
            return Err("SteamCMD exited with error".to_string());
        }
        Ok(())
    }
}