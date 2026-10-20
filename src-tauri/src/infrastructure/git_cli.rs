use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::settings::{
    NetworkPreferences, GIT_PROXY_MODE_DIRECT, GIT_PROXY_MODE_MANUAL, GIT_PROXY_MODE_SYSTEM,
};

const NETWORK_TIMEOUT: Duration = Duration::from_secs(120);
const LOCAL_TIMEOUT: Duration = Duration::from_secs(30);
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const GIT_INSTALL_URL: &str = "https://git-scm.com/download/win";

pub const GIT_PROXY_TEST_URL: &str = "https://github.com/obra/superpowers.git";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxySource {
    Manual,
    Environment,
    System,
    None,
    Pac,
}

impl ProxySource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Environment => "environment",
            Self::System => "system",
            Self::None => "none",
            Self::Pac => "pac",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyConfig {
    pub url: Option<String>,
    pub source: ProxySource,
    pub pac_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitEnvironmentReport {
    pub available: bool,
    pub version: Option<String>,
    pub executable: Option<String>,
    pub proxy: Option<String>,
    pub proxy_source: String,
    pub pac_detected: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitConnectionReport {
    pub success: bool,
    pub elapsed_ms: u64,
    pub proxy: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GitOptions {
    executable: PathBuf,
    proxy: ProxyConfig,
    mode: String,
}

impl GitOptions {
    pub fn resolve(preferences: &NetworkPreferences) -> DomainResult<Self> {
        let executable = discover_git_executable().ok_or_else(|| {
            DomainError::Git(format!(
                "未找到 Git。请先安装 Git for Windows：{GIT_INSTALL_URL}"
            ))
        })?;
        let proxy = resolve_proxy(preferences)?;
        Ok(Self {
            executable,
            proxy,
            mode: preferences.git_proxy_mode.clone(),
        })
    }

    pub fn inspect_environment(preferences: &NetworkPreferences) -> GitEnvironmentReport {
        let executable = discover_git_executable();
        let version = executable
            .as_deref()
            .and_then(|path| probe_git_version(path).ok());
        let available = executable.is_some() && version.is_some();
        let (proxy, error) = match resolve_proxy(preferences) {
            Ok(proxy) => (proxy, None),
            Err(error) => (
                ProxyConfig {
                    url: None,
                    source: ProxySource::None,
                    pac_detected: windows_pac_detected(),
                },
                Some(error.to_string()),
            ),
        };
        GitEnvironmentReport {
            available,
            version,
            executable: executable.map(|path| path.to_string_lossy().into_owned()),
            proxy: proxy.url,
            proxy_source: proxy.source.as_str().into(),
            pac_detected: proxy.pac_detected,
            error,
        }
    }

    pub fn test_connection(&self) -> GitConnectionReport {
        let started = Instant::now();
        match self.output(
            &["ls-remote", "--symref", GIT_PROXY_TEST_URL, "HEAD"],
            None,
            true,
        ) {
            Ok(output) if output.status.success() => GitConnectionReport {
                success: true,
                elapsed_ms: elapsed_ms(started),
                proxy: self.proxy.url.clone(),
                message: None,
            },
            Ok(output) => GitConnectionReport {
                success: false,
                elapsed_ms: elapsed_ms(started),
                proxy: self.proxy.url.clone(),
                message: Some(describe_failure(&output)),
            },
            Err(error) => GitConnectionReport {
                success: false,
                elapsed_ms: elapsed_ms(started),
                proxy: self.proxy.url.clone(),
                message: Some(error.to_string()),
            },
        }
    }

    /// Runs git with captured output and a bounded execution time.
    pub fn output<S: AsRef<OsStr>>(
        &self,
        args: &[S],
        cwd: Option<&Path>,
        network: bool,
    ) -> DomainResult<Output> {
        let mut command = self.command(network);
        if let Some(cwd) = cwd {
            command.arg("-C").arg(cwd);
        }
        command.args(args);
        let timeout = if network {
            NETWORK_TIMEOUT
        } else {
            LOCAL_TIMEOUT
        };
        let raw = run_with_timeout(command, timeout).map_err(|error| {
            DomainError::Git(format!("无法启动 Git：{error}。{GIT_INSTALL_URL}"))
        })?;
        if raw.timed_out {
            return Err(DomainError::Git(format!(
                "Git 操作超时（{} 秒），请检查网络或代理设置。",
                timeout.as_secs()
            )));
        }
        Ok(Output {
            status: raw.status,
            stdout: raw.stdout,
            stderr: raw.stderr,
        })
    }

    /// Runs git and converts a non-zero exit status into a detailed Git error.
    pub fn output_checked<S: AsRef<OsStr>>(
        &self,
        args: &[S],
        cwd: Option<&Path>,
        network: bool,
    ) -> DomainResult<Output> {
        let output = self.output(args, cwd, network)?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(DomainError::Git(describe_failure(&output)))
        }
    }

    fn command(&self, network: bool) -> Command {
        let mut command = Command::new(&self.executable);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
            command.args(["-c", "core.longpaths=true"]);
        }
        if !network {
            return command;
        }

        command.env("GIT_TERMINAL_PROMPT", "0");
        command.env("GCM_INTERACTIVE", "Never");
        match self.proxy.url.as_deref() {
            Some(url) => {
                for key in [
                    "HTTP_PROXY",
                    "http_proxy",
                    "HTTPS_PROXY",
                    "https_proxy",
                    "ALL_PROXY",
                    "all_proxy",
                ] {
                    command.env(key, url);
                }
                if self.mode == GIT_PROXY_MODE_MANUAL {
                    let http_proxy = format!("http.proxy={url}");
                    let https_proxy = format!("https.proxy={url}");
                    command.args(["-c", &http_proxy, "-c", &https_proxy]);
                }
            }
            None if self.mode == GIT_PROXY_MODE_DIRECT => {
                for key in [
                    "HTTP_PROXY",
                    "http_proxy",
                    "HTTPS_PROXY",
                    "https_proxy",
                    "ALL_PROXY",
                    "all_proxy",
                ] {
                    command.env_remove(key);
                }
                command.args(["-c", "http.proxy=", "-c", "https.proxy="]);
            }
            None => {}
        }
        command.args([
            "-c",
            "http.lowSpeedLimit=1000",
            "-c",
            "http.lowSpeedTime=30",
        ]);
        command
    }
}

/// Normalizes user input into a URL that `git clone` can actually fetch.
pub fn canonical_clone_url(input: &str) -> DomainResult<String> {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(DomainError::Git("Git 仓库地址不能为空".into()));
    }
    if trimmed.contains(char::is_whitespace) {
        return Err(DomainError::Git("Git 仓库地址不能包含空格".into()));
    }
    for scheme in ["https://", "http://", "ssh://", "git://"] {
        if let Some(rest) = trimmed.strip_prefix(scheme) {
            if rest.is_empty() {
                return Err(DomainError::Git("Git 仓库地址无效".into()));
            }
            return Ok(trimmed.to_string());
        }
    }
    if trimmed.starts_with("git@") {
        return Ok(trimmed.to_string());
    }
    if let Some((host, path)) = trimmed.split_once(':') {
        if host.contains('.') && path.split('/').count() >= 2 {
            return Ok(format!("https://{host}/{path}"));
        }
    }
    let first = trimmed.split('/').next().unwrap_or_default();
    if first.contains('.') && trimmed.split('/').count() >= 3 {
        return Ok(format!("https://{trimmed}"));
    }
    Err(DomainError::Git(format!(
        "Git 仓库地址无效：{input}。请使用 https://github.com/owner/repo 或 github.com/owner/repo"
    )))
}

pub fn validate_manual_proxy_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("手动代理模式下必须填写代理地址".into());
    }
    if trimmed.contains(char::is_whitespace) {
        return Err("代理地址不能包含空格".into());
    }
    if trimmed.contains('@') {
        return Err("暂不支持带用户名或密码的代理地址".into());
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };
    let (scheme, rest) = with_scheme
        .split_once("://")
        .ok_or_else(|| "代理地址格式无效".to_string())?;
    let scheme = scheme.to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https" | "socks5" | "socks5h") {
        return Err("代理地址仅支持 http、https、socks5".into());
    }
    if rest.is_empty() || rest.contains('/') {
        return Err("代理地址应形如 http://127.0.0.1:7890".into());
    }
    let (host, port) = rest
        .rsplit_once(':')
        .ok_or_else(|| "代理地址必须包含端口".to_string())?;
    if host.is_empty() || port.parse::<u16>().is_err() {
        return Err("代理地址必须包含有效端口".into());
    }
    Ok(with_scheme)
}

pub fn resolve_proxy(preferences: &NetworkPreferences) -> DomainResult<ProxyConfig> {
    let pac_detected = windows_pac_detected();
    match preferences.git_proxy_mode.as_str() {
        GIT_PROXY_MODE_MANUAL => {
            let url = validate_manual_proxy_url(&preferences.git_proxy_url)
                .map_err(|error| DomainError::Git(error))?;
            Ok(ProxyConfig {
                url: Some(url),
                source: ProxySource::Manual,
                pac_detected,
            })
        }
        GIT_PROXY_MODE_DIRECT => Ok(ProxyConfig {
            url: None,
            source: ProxySource::None,
            pac_detected,
        }),
        GIT_PROXY_MODE_SYSTEM | "" => {
            if let Some(url) = environment_proxy() {
                return Ok(ProxyConfig {
                    url: Some(url),
                    source: ProxySource::Environment,
                    pac_detected,
                });
            }
            #[cfg(target_os = "windows")]
            if let Some(url) = windows_system_proxy() {
                return Ok(ProxyConfig {
                    url: Some(url),
                    source: ProxySource::System,
                    pac_detected,
                });
            }
            Ok(ProxyConfig {
                url: None,
                source: if pac_detected {
                    ProxySource::Pac
                } else {
                    ProxySource::None
                },
                pac_detected,
            })
        }
        other => Err(DomainError::Git(format!("不支持的 Git 代理模式：{other}"))),
    }
}

fn environment_proxy() -> Option<String> {
    for key in [
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn windows_internet_settings() -> Option<winreg::RegKey> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            KEY_READ,
        )
        .ok()
}

#[cfg(target_os = "windows")]
fn windows_system_proxy() -> Option<String> {
    let settings = windows_internet_settings()?;
    let enabled = settings
        .get_value::<u32, _>("ProxyEnable")
        .map(|value| value != 0)
        .or_else(|_| {
            settings
                .get_value::<String, _>("ProxyEnable")
                .map(|value| value.trim() == "1")
        })
        .ok()?;
    if !enabled {
        return None;
    }
    let server: String = settings.get_value("ProxyServer").ok()?;
    parse_proxy_server(&server)
}

#[cfg(target_os = "windows")]
fn windows_pac_detected() -> bool {
    windows_internet_settings()
        .and_then(|settings| settings.get_value::<String, _>("AutoConfigURL").ok())
        .is_some_and(|value| !value.trim().is_empty())
}

#[cfg(not(target_os = "windows"))]
fn windows_pac_detected() -> bool {
    false
}

fn parse_proxy_server(server: &str) -> Option<String> {
    let server = server.trim();
    if server.is_empty() {
        return None;
    }
    if server.contains('=') {
        for entry in server.split(';') {
            let Some((scheme, address)) = entry.split_once('=') else {
                continue;
            };
            let address = address.trim();
            if address.is_empty() {
                continue;
            }
            match scheme.trim().to_ascii_lowercase().as_str() {
                "https" | "http" => return Some(with_proxy_scheme(address, "http://")),
                "socks" | "socks5" | "socks5h" => {
                    return Some(with_proxy_scheme(address, "socks5://"))
                }
                _ => {}
            }
        }
        None
    } else {
        Some(with_proxy_scheme(server, "http://"))
    }
}

fn with_proxy_scheme(value: &str, default_scheme: &str) -> String {
    let value = value.trim();
    if value.contains("://") {
        value.to_string()
    } else {
        format!("{default_scheme}{value}")
    }
}

fn discover_git_executable() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PATH") {
        for directory in std::env::split_paths(&path) {
            for name in git_file_names() {
                let candidate = directory.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        for candidate in windows_git_candidates() {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn git_file_names() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["git.exe", "git"]
    }
    #[cfg(not(target_os = "windows"))]
    {
        &["git"]
    }
}

#[cfg(target_os = "windows")]
fn windows_git_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut roots = Vec::new();
    for key in [
        "ProgramFiles",
        "ProgramFiles(x86)",
        "LOCALAPPDATA",
        "USERPROFILE",
    ] {
        if let Ok(root) = std::env::var(key) {
            roots.push(PathBuf::from(root));
        }
    }
    for root in [r"C:\Program Files\Git", r"C:\Program Files (x86)\Git"] {
        candidates.push(PathBuf::from(root).join("cmd").join("git.exe"));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local)
                .join("Programs")
                .join("Git")
                .join("cmd")
                .join("git.exe"),
        );
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        candidates.push(
            PathBuf::from(profile)
                .join("scoop")
                .join("apps")
                .join("git")
                .join("current")
                .join("cmd")
                .join("git.exe"),
        );
    }
    for root in roots {
        candidates.push(root.join("Git").join("cmd").join("git.exe"));
    }
    candidates
}

fn probe_git_version(executable: &Path) -> Result<String, DomainError> {
    let mut command = Command::new(executable);
    command.arg("--version");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let raw = run_with_timeout(command, PROBE_TIMEOUT)
        .map_err(|error| DomainError::Git(format!("无法启动 Git：{error}")))?;
    if raw.timed_out {
        return Err(DomainError::Git("Git 版本检测超时".into()));
    }
    if !raw.status.success() {
        return Err(DomainError::Git(
            String::from_utf8_lossy(&raw.stderr).trim().to_string(),
        ));
    }
    let version = String::from_utf8_lossy(&raw.stdout).trim().to_string();
    if version.is_empty() {
        return Err(DomainError::Git("Git 版本信息为空".into()));
    }
    Ok(version
        .strip_prefix("git version ")
        .unwrap_or(&version)
        .to_string())
}

struct RawOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
}

fn run_with_timeout(mut command: Command, timeout: Duration) -> std::io::Result<RawOutput> {
    let id = uuid::Uuid::new_v4();
    let directory = std::env::temp_dir();
    let stdout_path = directory.join(format!("agentpalette-git-{id}.out"));
    let stderr_path = directory.join(format!("agentpalette-git-{id}.err"));
    let stdout_file = std::fs::File::create(&stdout_path)?;
    let stderr_file = std::fs::File::create(&stderr_path)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    let mut child = command.spawn()?;
    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if started.elapsed() >= timeout => {
                timed_out = true;
                let _ = child.kill();
                break child.wait()?;
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    };

    let stdout = std::fs::read(&stdout_path).unwrap_or_default();
    let stderr = std::fs::read(&stderr_path).unwrap_or_default();
    let _ = std::fs::remove_file(&stdout_path);
    let _ = std::fs::remove_file(&stderr_path);
    Ok(RawOutput {
        status,
        stdout,
        stderr,
        timed_out,
    })
}

fn describe_failure(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail = stderr_tail(&stderr, 12);
    let mut message = if tail.is_empty() {
        format!("Git 退出码：{}", output.status)
    } else {
        tail
    };
    if let Some(hint) = failure_hint(&stderr) {
        message.push('\n');
        message.push_str(hint);
    }
    message
}

fn stderr_tail(stderr: &str, max_lines: usize) -> String {
    let lines = stderr
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    lines
        .iter()
        .skip(lines.len().saturating_sub(max_lines))
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
}

fn failure_hint(stderr: &str) -> Option<&'static str> {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("could not resolve host")
        || lower.contains("connection was reset")
        || lower.contains("failed to connect")
        || lower.contains("unable to access")
        || lower.contains("operation timed out")
        || lower.contains("connection timed out")
        || lower.contains("recv failure")
        || lower.contains("send failure")
    {
        return Some("无法连接远端仓库：请检查网络连接，或在设置 > 网络与 Git 中配置代理。");
    }
    if lower.contains("does not appear to be a git repository")
        || lower.contains("repository not found")
        || lower.contains("not found")
    {
        return Some("仓库不存在或地址无效；Git 导入仅支持公开仓库。");
    }
    if lower.contains("permission denied (publickey)") {
        return Some(
            "SSH 方式需要本机已配置 GitHub SSH 密钥；建议改用 https://github.com/owner/repo 地址。",
        );
    }
    if lower.contains("authentication failed")
        || lower.contains("could not read username")
        || lower.contains("terminal prompts disabled")
    {
        return Some("Git 导入目前仅支持公开仓库，私有仓库需要凭据。");
    }
    if lower.contains("filename too long") {
        return Some("仓库路径过长：已在 Windows 上启用 core.longpaths，请重试。");
    }
    None
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_clone_url, parse_proxy_server, validate_manual_proxy_url, GitOptions, ProxySource,
    };
    use crate::domain::settings::{
        NetworkPreferences, GIT_PROXY_MODE_DIRECT, GIT_PROXY_MODE_MANUAL,
    };
    use crate::infrastructure::git_cli::resolve_proxy;

    #[test]
    fn parses_plain_and_per_protocol_windows_proxies() {
        assert_eq!(
            parse_proxy_server("127.0.0.1:7890"),
            Some("http://127.0.0.1:7890".into())
        );
        assert_eq!(
            parse_proxy_server("http=127.0.0.1:7890;https=127.0.0.1:7891"),
            Some("http://127.0.0.1:7890".into())
        );
        assert_eq!(
            parse_proxy_server("socks=127.0.0.1:7890"),
            Some("socks5://127.0.0.1:7890".into())
        );
        assert_eq!(
            parse_proxy_server("http://127.0.0.1:7890"),
            Some("http://127.0.0.1:7890".into())
        );
        assert_eq!(parse_proxy_server("  "), None);
    }

    #[test]
    fn canonicalizes_scheme_less_github_urls() {
        assert_eq!(
            canonical_clone_url("github.com/obra/superpowers").unwrap(),
            "https://github.com/obra/superpowers"
        );
        assert_eq!(
            canonical_clone_url("github.com:obra/superpowers.git").unwrap(),
            "https://github.com/obra/superpowers.git"
        );
        assert_eq!(
            canonical_clone_url("https://github.com/obra/superpowers.git").unwrap(),
            "https://github.com/obra/superpowers.git"
        );
        assert_eq!(
            canonical_clone_url("git@github.com:obra/superpowers.git").unwrap(),
            "git@github.com:obra/superpowers.git"
        );
        assert!(canonical_clone_url("not a url").is_err());
        assert!(canonical_clone_url("obra/superpowers").is_err());
    }

    #[test]
    fn validates_manual_proxy_urls() {
        assert_eq!(
            validate_manual_proxy_url("127.0.0.1:7890").unwrap(),
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            validate_manual_proxy_url("socks5://127.0.0.1:7890").unwrap(),
            "socks5://127.0.0.1:7890"
        );
        assert!(validate_manual_proxy_url("").is_err());
        assert!(validate_manual_proxy_url("http://user:pass@127.0.0.1:7890").is_err());
        assert!(validate_manual_proxy_url("http://127.0.0.1").is_err());
        assert!(validate_manual_proxy_url("ftp://127.0.0.1:7890").is_err());
    }

    #[test]
    fn direct_mode_never_uses_a_proxy() {
        let preferences = NetworkPreferences {
            git_proxy_mode: GIT_PROXY_MODE_DIRECT.into(),
            git_proxy_url: "http://127.0.0.1:7890".into(),
        };
        let proxy = resolve_proxy(&preferences).unwrap();
        assert_eq!(proxy.url, None);
        assert_eq!(proxy.source, ProxySource::None);
    }

    #[test]
    fn manual_mode_uses_the_configured_proxy() {
        let preferences = NetworkPreferences {
            git_proxy_mode: GIT_PROXY_MODE_MANUAL.into(),
            git_proxy_url: "127.0.0.1:7890".into(),
        };
        let proxy = resolve_proxy(&preferences).unwrap();
        assert_eq!(proxy.url.as_deref(), Some("http://127.0.0.1:7890"));
        assert_eq!(proxy.source, ProxySource::Manual);
    }

    #[test]
    #[ignore = "requires network access and a reachable GitHub endpoint"]
    fn connects_to_github_with_the_resolved_proxy() {
        let options = GitOptions::resolve(&NetworkPreferences::default()).unwrap();
        let report = options.test_connection();
        assert!(report.success, "{:?}", report.message);
    }
}
