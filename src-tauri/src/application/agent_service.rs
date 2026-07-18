use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::settings::LaunchPreferences;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalAgent {
    pub id: String,
    pub product: String,
    pub display_name: String,
    pub surface: String,
    pub command: String,
    pub status: String,
    pub version: Option<String>,
    pub executable_path: Option<String>,
    pub can_install: bool,
    pub can_update: bool,
    pub can_uninstall: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUpdate {
    pub agent_id: String,
    pub status: String,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMaintenancePlan {
    pub agent_id: String,
    pub action: String,
    pub command: String,
}

struct AgentDefinition {
    id: &'static str,
    product: &'static str,
    display_name: &'static str,
    commands: &'static [&'static str],
    maintenance: AgentMaintenance,
}

#[derive(Clone, Copy)]
enum AgentMaintenance {
    Npm(&'static str),
    Antigravity,
    Hermes,
    Cursor,
}

const AGENTS: &[AgentDefinition] = &[
    AgentDefinition {
        id: "codex",
        product: "Codex",
        display_name: "Codex CLI",
        commands: &["codex"],
        maintenance: AgentMaintenance::Npm("@openai/codex"),
    },
    AgentDefinition {
        id: "claude",
        product: "Claude",
        display_name: "Claude Code",
        commands: &["claude"],
        maintenance: AgentMaintenance::Npm("@anthropic-ai/claude-code"),
    },
    AgentDefinition {
        id: "antigravity",
        product: "Antigravity",
        display_name: "Antigravity CLI",
        commands: &["agy"],
        maintenance: AgentMaintenance::Antigravity,
    },
    AgentDefinition {
        id: "gemini",
        product: "Gemini",
        display_name: "Gemini CLI",
        commands: &["gemini"],
        maintenance: AgentMaintenance::Npm("@google/gemini-cli"),
    },
    AgentDefinition {
        id: "opencode",
        product: "OpenCode",
        display_name: "OpenCode",
        commands: &["opencode"],
        maintenance: AgentMaintenance::Npm("opencode-ai"),
    },
    AgentDefinition {
        id: "openclaw",
        product: "OpenClaw",
        display_name: "OpenClaw CLI",
        commands: &["openclaw"],
        maintenance: AgentMaintenance::Npm("openclaw"),
    },
    AgentDefinition {
        id: "hermes",
        product: "Hermes",
        display_name: "Hermes Agent",
        commands: &["hermes"],
        maintenance: AgentMaintenance::Hermes,
    },
    AgentDefinition {
        id: "cursor",
        product: "Cursor",
        display_name: "Cursor CLI",
        commands: &["cursor-agent"],
        maintenance: AgentMaintenance::Cursor,
    },
];

pub struct AgentService;

impl Default for AgentService {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentService {
    pub fn new() -> Self {
        Self
    }

    pub fn discover(&self) -> Vec<LocalAgent> {
        let mut agents: Vec<LocalAgent> = AGENTS
            .iter()
            .map(|definition| {
                let executable = definition
                    .commands
                    .iter()
                    .find_map(|command| find_executable(command));
                let version = executable
                    .as_deref()
                    .and_then(read_version)
                    .and_then(normalize_version);
                LocalAgent {
                    id: definition.id.into(),
                    product: definition.product.into(),
                    display_name: definition.display_name.into(),
                    surface: "cli".into(),
                    command: definition.commands[0].into(),
                    status: if executable.is_some() {
                        "ready".into()
                    } else {
                        "missing".into()
                    },
                    version,
                    executable_path: executable.map(|path| path.to_string_lossy().to_string()),
                    can_install: can_maintain(definition, "install"),
                    can_update: can_maintain(definition, "update"),
                    can_uninstall: can_maintain(definition, "uninstall"),
                }
            })
            .collect();
        agents.extend(discover_desktop_agents());
        agents
    }

    pub fn launch(
        &self,
        agent_id: &str,
        project_path: &str,
        preferences: &LaunchPreferences,
    ) -> DomainResult<()> {
        let definition = AGENTS
            .iter()
            .find(|agent| agent.id == agent_id)
            .ok_or_else(|| DomainError::Operation("未知 Agent。".into()))?;
        let executable = definition
            .commands
            .iter()
            .find_map(|command| find_executable(command))
            .ok_or_else(|| {
                DomainError::Operation(format!("未找到 {}。", definition.display_name))
            })?;
        let project = Path::new(project_path);
        if !project.is_dir() {
            return Err(DomainError::Operation("项目目录不存在或不可访问。".into()));
        }
        launch_in_terminal(&executable, project, preferences)
    }

    pub fn open_desktop(&self, agent_id: &str) -> DomainResult<()> {
        let agent = discover_desktop_agents()
            .into_iter()
            .find(|agent| agent.id == agent_id && agent.status == "ready")
            .ok_or_else(|| DomainError::Operation("未发现该桌面 Agent。".into()))?;
        let path = agent
            .executable_path
            .ok_or_else(|| DomainError::Operation("桌面 Agent 缺少应用路径。".into()))?;
        open_desktop_application(Path::new(&path))
    }

    pub fn check_updates(&self) -> Vec<AgentUpdate> {
        AGENTS
            .iter()
            .map(|definition| {
                let executable = definition
                    .commands
                    .iter()
                    .find_map(|command| find_executable(command));
                let current = executable
                    .as_deref()
                    .and_then(read_version)
                    .and_then(normalize_version);

                let latest = match definition.maintenance {
                    AgentMaintenance::Npm(package) => read_npm_latest_version(package),
                    _ => None,
                };
                create_update(definition, current, latest)
            })
            .collect()
    }

    pub fn maintenance_plan(
        &self,
        agent_id: &str,
        action: &str,
    ) -> DomainResult<AgentMaintenancePlan> {
        let definition = managed_agent(agent_id, action)?;
        let command = maintenance_command(definition, action)?;
        Ok(AgentMaintenancePlan {
            agent_id: definition.id.into(),
            action: action.into(),
            command,
        })
    }

    pub fn apply_maintenance(&self, agent_id: &str, action: &str) -> DomainResult<()> {
        let definition = managed_agent(agent_id, action)?;
        let output = run_maintenance(definition, action)?;
        if output.status.success() {
            Ok(())
        } else {
            let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(DomainError::Operation(if details.is_empty() {
                "npm 未能完成操作。".into()
            } else {
                format!("npm 未能完成操作：{details}")
            }))
        }
    }
}

fn create_update(
    definition: &AgentDefinition,
    current_version: Option<String>,
    latest_version: Option<String>,
) -> AgentUpdate {
    let status = match definition.maintenance {
        AgentMaintenance::Npm(_) => match (&current_version, &latest_version) {
            (None, _) => "not_installed",
            (Some(_), None) => "unknown",
            (Some(current), Some(latest)) if current == latest => "current",
            (Some(_), Some(_)) => "available",
        },
        _ if current_version.is_some() => "unknown",
        _ => "not_installed",
    };
    AgentUpdate {
        agent_id: definition.id.into(),
        status: status.into(),
        current_version,
        latest_version,
    }
}

fn managed_agent(agent_id: &str, action: &str) -> DomainResult<&'static AgentDefinition> {
    AGENTS
        .iter()
        .find(|agent| agent.id == agent_id && can_maintain(agent, action))
        .ok_or_else(|| DomainError::Operation("此 Agent 暂不支持该维护操作。".into()))
}

fn can_maintain(definition: &AgentDefinition, action: &str) -> bool {
    match definition.maintenance {
        AgentMaintenance::Npm(_) | AgentMaintenance::Hermes => {
            matches!(action, "install" | "update" | "uninstall")
        }
        AgentMaintenance::Antigravity => matches!(action, "install" | "update"),
        AgentMaintenance::Cursor => {
            !cfg!(target_os = "windows") && matches!(action, "install" | "update")
        }
    }
}

fn maintenance_command(definition: &AgentDefinition, action: &str) -> DomainResult<String> {
    let command = match definition.maintenance {
        AgentMaintenance::Npm(package) => match action {
            "install" | "update" => format!("npm install -g {package}@latest"),
            "uninstall" => format!("npm uninstall -g {package}"),
            _ => unreachable!(),
        },
        AgentMaintenance::Antigravity => match action {
            "install" | "update" if cfg!(target_os = "windows") => {
                "irm https://antigravity.google/cli/install.ps1 | iex".into()
            }
            "install" | "update" => {
                "curl -fsSL https://antigravity.google/cli/install.sh | bash".into()
            }
            _ => unreachable!(),
        },
        AgentMaintenance::Hermes => match action {
            "install" if cfg!(target_os = "windows") => {
                "iex (irm https://hermes-agent.nousresearch.com/install.ps1)".into()
            }
            "install" => {
                "curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash".into()
            }
            "update" => "hermes update".into(),
            "uninstall" => "hermes uninstall --yes".into(),
            _ => unreachable!(),
        },
        AgentMaintenance::Cursor => match action {
            "install" => "curl https://cursor.com/install -fsS | bash".into(),
            "update" => "cursor-agent update".into(),
            _ => unreachable!(),
        },
    };
    Ok(command)
}

fn run_maintenance(
    definition: &AgentDefinition,
    action: &str,
) -> DomainResult<std::process::Output> {
    match definition.maintenance {
        AgentMaintenance::Npm(package) => {
            let npm = find_executable("npm").ok_or_else(|| {
                DomainError::Operation("未找到 npm，无法管理此 Agent。请先安装 Node.js。".into())
            })?;
            let args: Vec<String> = match action {
                "install" | "update" => {
                    vec!["install".into(), "-g".into(), format!("{package}@latest")]
                }
                "uninstall" => vec!["uninstall".into(), "-g".into(), package.into()],
                _ => unreachable!(),
            };
            let mut command = Command::new(npm);
            command.args(&args);
            hide_console_window(&mut command);
            command
                .output()
                .map_err(|error| DomainError::Operation(format!("无法执行 npm：{error}")))
        }
        AgentMaintenance::Hermes if action != "install" => {
            let executable = find_executable("hermes")
                .ok_or_else(|| DomainError::Operation("未找到 Hermes CLI。".into()))?;
            let args = if action == "update" {
                vec!["update"]
            } else {
                vec!["uninstall", "--yes"]
            };
            let mut command = Command::new(executable);
            command.args(args);
            hide_console_window(&mut command);
            command
                .output()
                .map_err(|error| DomainError::Operation(format!("无法执行 Hermes：{error}")))
        }
        _ => run_platform_shell(&maintenance_command(definition, action)?),
    }
}

#[cfg(target_os = "windows")]
fn run_platform_shell(command: &str) -> DomainResult<std::process::Output> {
    let mut shell = Command::new("powershell");
    shell.args(["-NoProfile", "-Command", command]);
    hide_console_window(&mut shell);
    shell
        .output()
        .map_err(|error| DomainError::Operation(format!("无法执行维护命令：{error}")))
}

#[cfg(not(target_os = "windows"))]
fn run_platform_shell(command: &str) -> DomainResult<std::process::Output> {
    Command::new("sh")
        .args(["-c", command])
        .output()
        .map_err(|error| DomainError::Operation(format!("无法执行维护命令：{error}")))
}

fn normalize_version(raw: String) -> Option<String> {
    raw.split_whitespace().rev().find_map(|value| {
        let value = value
            .trim_start_matches('v')
            .trim_matches(|character: char| !character.is_ascii_digit() && character != '.');
        value
            .chars()
            .next()
            .filter(|character| character.is_ascii_digit())
            .map(|_| value.to_string())
    })
}

fn read_npm_latest_version(package: &str) -> Option<String> {
    let npm = find_executable("npm")?;
    let mut command = Command::new(npm);
    command.args([
        "view",
        package,
        "version",
        "--json",
        "--fetch-retries=0",
        "--fetch-timeout=3000",
    ]);
    hide_console_window(&mut command);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice::<String>(&output.stdout).ok()
}

#[cfg(target_os = "macos")]
fn discover_desktop_agents() -> Vec<LocalAgent> {
    [
        ("codex-desktop", "Codex", "ChatGPT Desktop", "ChatGPT.app"),
        (
            "antigravity-desktop",
            "Antigravity",
            "Antigravity Desktop",
            "Antigravity.app",
        ),
    ]
    .into_iter()
    .map(|(id, product, display_name, bundle)| {
        let path = [
            PathBuf::from("/Applications").join(bundle),
            dirs::home_dir()
                .unwrap_or_default()
                .join("Applications")
                .join(bundle),
        ]
        .into_iter()
        .find(|path| path.is_dir());
        LocalAgent {
            id: id.into(),
            product: product.into(),
            display_name: display_name.into(),
            surface: "desktop".into(),
            command: bundle.into(),
            status: if path.is_some() {
                "ready".into()
            } else {
                "missing".into()
            },
            version: path.as_deref().and_then(read_bundle_version),
            executable_path: path.map(|path| path.to_string_lossy().to_string()),
            can_install: false,
            can_update: false,
            can_uninstall: false,
        }
    })
    .collect()
}

#[cfg(target_os = "windows")]
fn discover_desktop_agents() -> Vec<LocalAgent> {
    let program_files = env::var_os("PROGRAMFILES")
        .map(PathBuf::from)
        .unwrap_or_default();
    let local_app_data = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_default();
    [
        (
            "codex-desktop",
            "Codex",
            "ChatGPT Desktop",
            vec![local_app_data.join("Programs/ChatGPT/ChatGPT.exe")],
        ),
        (
            "antigravity-desktop",
            "Antigravity",
            "Antigravity Desktop",
            vec![program_files.join("Antigravity/Antigravity.exe")],
        ),
    ]
    .into_iter()
    .map(|(id, product, display_name, candidates)| {
        let path = candidates.into_iter().find(|path| path.is_file());
        LocalAgent {
            id: id.into(),
            product: product.into(),
            display_name: display_name.into(),
            surface: "desktop".into(),
            command: "desktop".into(),
            status: if path.is_some() {
                "ready".into()
            } else {
                "missing".into()
            },
            version: None,
            executable_path: path.map(|path| path.to_string_lossy().to_string()),
            can_install: false,
            can_update: false,
            can_uninstall: false,
        }
    })
    .collect()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn discover_desktop_agents() -> Vec<LocalAgent> {
    Vec::new()
}

#[cfg(target_os = "macos")]
fn read_bundle_version(bundle: &Path) -> Option<String> {
    let plist = bundle.join("Contents/Info.plist");
    let output = Command::new("plutil")
        .args([
            "-extract",
            "CFBundleShortVersionString",
            "raw",
            &plist.to_string_lossy(),
        ])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|version| !version.is_empty())
}

#[cfg(target_os = "macos")]
fn open_desktop_application(path: &Path) -> DomainResult<()> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|error| DomainError::Operation(format!("无法打开桌面 Agent：{error}")))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_desktop_application(path: &Path) -> DomainResult<()> {
    Command::new(path)
        .spawn()
        .map_err(|error| DomainError::Operation(format!("无法打开桌面 Agent：{error}")))?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn open_desktop_application(_path: &Path) -> DomainResult<()> {
    Err(DomainError::Operation(
        "当前平台尚未支持桌面 Agent。".into(),
    ))
}

fn find_executable(command: &str) -> Option<PathBuf> {
    let mut directories: Vec<PathBuf> = env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).collect())
        .unwrap_or_default();
    if let Some(home) = dirs::home_dir() {
        directories.extend([
            home.join(".local/bin"),
            home.join(".npm-global/bin"),
            home.join(".cargo/bin"),
        ]);
    }
    if cfg!(target_os = "macos") {
        directories.push(PathBuf::from("/opt/homebrew/bin"));
    }
    if cfg!(target_os = "windows") {
        if let Some(app_data) = env::var_os("APPDATA") {
            directories.push(PathBuf::from(app_data).join("npm"));
        }
    }
    let extensions: &[&str] = if cfg!(target_os = "windows") {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };
    directories.into_iter().find_map(|directory| {
        extensions
            .iter()
            .map(|extension| directory.join(format!("{command}{extension}")))
            .find(|candidate| candidate.is_file())
    })
}

fn read_version(executable: &Path) -> Option<String> {
    let mut command = Command::new(executable);
    command.arg("--version");
    hide_console_window(&mut command);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .next()?
        .trim()
        .to_string();
    (!value.is_empty()).then_some(value)
}

fn run_command_with_timeout(command: &mut Command, timeout: Duration) -> Option<Output> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().ok()?;
    let started = Instant::now();
    loop {
        if child.try_wait().ok()?.is_some() {
            return child.wait_with_output().ok();
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait_with_output();
            return None;
        }
        thread::sleep(Duration::from_millis(25));
    }
}

/// Prevents background CLI probes from creating a visible `cmd.exe` or PowerShell window.
///
/// This is deliberately not applied to `launch_in_terminal`, where a visible terminal is the
/// requested user-facing behavior.
#[cfg(target_os = "windows")]
fn hide_console_window(command: &mut Command) {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window(_command: &mut Command) {}

#[cfg(target_os = "macos")]
fn launch_in_terminal(
    executable: &Path,
    project: &Path,
    preferences: &LaunchPreferences,
) -> DomainResult<()> {
    let command = format!(
        "cd -- {} && exec {}",
        shell_quote(&project.to_string_lossy()),
        shell_quote(&executable.to_string_lossy())
    );
    let script = match preferences.macos_terminal.as_str() {
        "auto" | "terminal" => format!("tell application \"Terminal\" to do script \"{}\"", applescript_quote(&command)),
        "iterm" => format!(
            "tell application \"iTerm\"\nactivate\ncreate window with default profile\ntell current session of current window\nwrite text \"{}\"\nend tell\nend tell",
            applescript_quote(&command)
        ),
        other => return Err(DomainError::Operation(format!("当前版本尚不支持通过 {other} 自动启动，请选择 Terminal 或 iTerm2。"))),
    };
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|error| DomainError::Operation(format!("无法打开终端：{error}")))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn launch_in_terminal(
    executable: &Path,
    project: &Path,
    preferences: &LaunchPreferences,
) -> DomainResult<()> {
    let project = project.to_string_lossy();
    let executable = executable.to_string_lossy();
    let mut command = match preferences.windows_terminal.as_str() {
        "auto" if find_executable("wt").is_some() => {
            let mut value = Command::new("wt");
            value.args(["-d", &project, "cmd", "/K", &executable]);
            value
        }
        "windows_terminal" => {
            let mut value = Command::new("wt");
            value.args(["-d", &project, "cmd", "/K", &executable]);
            value
        }
        "auto" | "powershell" => {
            let mut value = Command::new("powershell");
            value.args([
                "-NoExit",
                "-Command",
                &format!(
                    "Set-Location -LiteralPath '{}'; & '{}'",
                    project.replace('\'', "''"),
                    executable.replace('\'', "''")
                ),
            ]);
            value
        }
        other => {
            return Err(DomainError::Operation(format!(
                "当前版本尚不支持通过 {other} 自动启动，请选择 Windows Terminal 或 PowerShell。"
            )))
        }
    };
    command
        .spawn()
        .map_err(|error| DomainError::Operation(format!("无法打开终端：{error}")))?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn launch_in_terminal(
    _executable: &Path,
    _project: &Path,
    _preferences: &LaunchPreferences,
) -> DomainResult<()> {
    Err(DomainError::Operation(
        "当前平台尚未支持 Agent 启动。".into(),
    ))
}

#[cfg(target_os = "macos")]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\\"'\\\"'"))
}

#[cfg(target_os = "macos")]
fn applescript_quote(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{create_update, normalize_version, AgentMaintenance, AgentService, AGENTS};

    #[test]
    fn discovery_includes_the_supported_agent_catalog() {
        let agents = AgentService::new().discover();
        assert_eq!(
            agents.iter().filter(|agent| agent.surface == "cli").count(),
            8
        );
        assert!(agents.iter().any(|agent| agent.id == "codex"));
        assert!(agents.iter().any(|agent| agent.command == "agy"));
        assert!(agents.iter().any(|agent| agent.command == "openclaw"));
        assert!(agents.iter().any(|agent| agent.command == "hermes"));
        assert!(agents.iter().any(|agent| agent.command == "cursor-agent"));
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        assert!(agents.iter().any(|agent| agent.id == "antigravity-desktop"));

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        assert!(!agents.iter().any(|agent| agent.surface == "desktop"));
    }

    #[test]
    fn maintenance_plans_use_only_supported_agent_actions() {
        let plan = AgentService::new()
            .maintenance_plan("codex", "update")
            .unwrap();
        assert_eq!(plan.command, "npm install -g @openai/codex@latest");
        assert_eq!(
            AgentService::new()
                .maintenance_plan("hermes", "update")
                .unwrap()
                .command,
            "hermes update"
        );
        assert!(AgentService::new()
            .maintenance_plan("cursor", "uninstall")
            .is_err());
    }

    #[test]
    fn version_normalization_handles_cli_prefixes() {
        assert_eq!(
            normalize_version("codex-cli 0.42.0".into()).as_deref(),
            Some("0.42.0")
        );
        assert_eq!(normalize_version("v1.2.3".into()).as_deref(), Some("1.2.3"));
    }

    #[test]
    fn non_npm_agents_return_a_terminal_update_status() {
        let antigravity = AGENTS
            .iter()
            .find(|agent| agent.id == "antigravity")
            .unwrap();
        let installed = create_update(antigravity, Some("1.2.3".into()), None);
        assert_eq!(installed.status, "unknown");
        assert_eq!(installed.current_version.as_deref(), Some("1.2.3"));

        assert_eq!(
            create_update(antigravity, None, None).status,
            "not_installed"
        );
    }

    #[test]
    fn npm_agents_compare_current_and_latest_versions() {
        let npm_agent = AGENTS
            .iter()
            .find(|agent| matches!(agent.maintenance, AgentMaintenance::Npm(_)))
            .unwrap();
        assert_eq!(
            create_update(npm_agent, Some("1.2.3".into()), Some("1.2.3".into())).status,
            "current"
        );
        assert_eq!(
            create_update(npm_agent, Some("1.2.3".into()), Some("1.2.4".into())).status,
            "available"
        );
    }
}
