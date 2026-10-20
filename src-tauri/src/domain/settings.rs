use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreferences {
    pub macos_terminal: String,
    pub windows_terminal: String,
    pub launch_presentation: String,
    pub show_command_preview: bool,
    pub check_environment: bool,
    pub check_permissions: bool,
    pub allow_copy_command_fallback: bool,
}

impl Default for LaunchPreferences {
    fn default() -> Self {
        Self {
            macos_terminal: "auto".into(),
            windows_terminal: "auto".into(),
            launch_presentation: "new_tab".into(),
            show_command_preview: true,
            check_environment: true,
            check_permissions: true,
            allow_copy_command_fallback: true,
        }
    }
}

pub const GIT_PROXY_MODE_SYSTEM: &str = "system";
pub const GIT_PROXY_MODE_DIRECT: &str = "direct";
pub const GIT_PROXY_MODE_MANUAL: &str = "manual";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPreferences {
    pub git_proxy_mode: String,
    pub git_proxy_url: String,
}

impl Default for NetworkPreferences {
    fn default() -> Self {
        Self {
            git_proxy_mode: GIT_PROXY_MODE_SYSTEM.into(),
            git_proxy_url: String::new(),
        }
    }
}
