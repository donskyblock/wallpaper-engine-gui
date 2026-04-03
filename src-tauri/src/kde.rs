use directories::ProjectDirs;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const PLUGIN_REPO_URL: &str = "https://github.com/catsout/wallpaper-engine-kde-plugin.git";

#[derive(Clone, Serialize)]
pub struct PluginState {
    pub supported: bool,
    pub plugin_installed: bool,
    pub desktop: String,
    pub repo_dir: String,
    pub missing_tools: Vec<String>,
    pub message: String,
}

pub fn plugin_state() -> Result<PluginState, String> {
    let desktop = crate::desktop::current_desktop_name();
    let repo_dir = plugin_workspace_dir()?;
    let missing_tools = required_tools()
        .into_iter()
        .filter(|tool| !command_available(tool))
        .collect::<Vec<_>>();

    let plugin_installed = installed_plugin_paths().iter().any(|path| path.exists());

    Ok(PluginState {
        supported: true,
        plugin_installed,
        desktop,
        repo_dir: repo_dir.to_string_lossy().into_owned(),
        missing_tools,
        message: if plugin_installed {
            "KDE Plasma support is installed and ready.".into()
        } else {
            "KDE Plasma was detected, but the Wallpaper Engine plugin is not installed.".into()
        },
    })
}

pub fn install_plugin() -> Result<PluginState, String> {
    let state = plugin_state()?;

    if !state.missing_tools.is_empty() {
        return Err(format!(
            "Missing required tools: {}",
            state.missing_tools.join(", ")
        ));
    }

    let repo_dir = PathBuf::from(&state.repo_dir);
    let build_dir = repo_dir.join("build");
    fs::create_dir_all(repo_dir.parent().unwrap_or(&repo_dir)).map_err(|e| e.to_string())?;

    if repo_dir.exists() {
        run_command(
            Command::new("git").args(["-C", &repo_dir.to_string_lossy(), "pull", "--ff-only"]),
            "Updating the KDE plugin repository failed",
        )?;
    } else {
        run_command(
            Command::new("git").args(["clone", PLUGIN_REPO_URL, &repo_dir.to_string_lossy()]),
            "Cloning the KDE plugin repository failed",
        )?;
    }

    fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;

    run_command(
        Command::new("cmake")
            .args(["-DUSE_PLASMAPKG=ON", ".."])
            .current_dir(&build_dir),
        "Configuring the KDE plugin build failed",
    )?;

    run_command(
        Command::new("cmake")
            .args(["--build", ".", "--", "-j4"])
            .current_dir(&build_dir),
        "Building the KDE plugin failed",
    )?;

    run_command(
        Command::new("cmake")
            .args(["--install", "."])
            .current_dir(&build_dir),
        "Installing the KDE plugin failed",
    )?;

    plugin_state()
}

fn installed_plugin_paths() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    vec![
        PathBuf::from(&home)
            .join(".local/share/plasma/wallpapers/com.github.casout.wallpaperEngineKde"),
        PathBuf::from("/usr/share/plasma/wallpapers/com.github.casout.wallpaperEngineKde"),
    ]
}

fn plugin_workspace_dir() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from("online", "opencom", "wallpaper-engine-linux")
        .ok_or_else(|| "Failed to resolve the app data directory.".to_string())?;
    Ok(dirs.data_local_dir().join("kde-plugin"))
}

fn command_available(tool: &str) -> bool {
    Command::new(tool).arg("--version").output().is_ok()
}

fn required_tools() -> Vec<String> {
    vec!["git".into(), "cmake".into()]
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
