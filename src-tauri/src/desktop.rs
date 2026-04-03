use serde::Serialize;

use crate::kde;

#[derive(Clone, Serialize)]
pub struct DesktopState {
    pub backend_id: String,
    pub backend_name: String,
    pub detected_session: String,
    pub supported: bool,
    pub plugin_installed: bool,
    pub plugin_workspace: Option<String>,
    pub missing_tools: Vec<String>,
    pub message: String,
}

pub enum DesktopBackend {
    Kde,
    Unsupported(String),
}

impl DesktopBackend {
    pub fn detect() -> Self {
        let session = current_desktop_name().to_lowercase();

        if session.contains("kde") || session.contains("plasma") {
            Self::Kde
        } else {
            Self::Unsupported(current_desktop_name())
        }
    }

    pub fn state(&self) -> Result<DesktopState, String> {
        match self {
            Self::Kde => {
                let plugin = kde::plugin_state()?;
                Ok(DesktopState {
                    backend_id: "kde".into(),
                    backend_name: "KDE Plasma".into(),
                    detected_session: plugin.desktop.clone(),
                    supported: plugin.supported,
                    plugin_installed: plugin.plugin_installed,
                    plugin_workspace: Some(plugin.repo_dir.clone()),
                    missing_tools: plugin.missing_tools.clone(),
                    message: plugin.message,
                })
            }
            Self::Unsupported(session) => Ok(DesktopState {
                backend_id: "unsupported".into(),
                backend_name: "Unsupported Desktop".into(),
                detected_session: session.clone(),
                supported: false,
                plugin_installed: false,
                plugin_workspace: None,
                missing_tools: Vec::new(),
                message: format!(
                    "{session} was detected. KDE Plasma is currently implemented, and the backend layer is ready for future non-GNOME targets."
                ),
            }),
        }
    }

    pub fn install_plugin(&self) -> Result<DesktopState, String> {
        match self {
            Self::Kde => {
                kde::install_plugin()?;
                self.state()
            }
            Self::Unsupported(_) => Err(
                "Plugin installation is only available for the KDE Plasma backend right now."
                    .into(),
            ),
        }
    }
}

pub fn current_desktop_name() -> String {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    if !desktop.trim().is_empty() {
        return desktop;
    }

    let session = std::env::var("DESKTOP_SESSION").unwrap_or_default();
    if !session.trim().is_empty() {
        return session;
    }

    "Unknown".into()
}
