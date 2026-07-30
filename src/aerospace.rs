use serde::de::DeserializeOwned;
use std::{fmt::format, process::Command};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AerospaceApp {
    #[serde(rename = "app-bundle-id")]
    pub app_bundle_id: String,
    #[serde(rename = "app-name")]
    pub app_name: String,
    #[serde(rename = "app-pid")]
    pub app_pid: i32,
}

#[derive(Debug, Deserialize)]
pub struct AerospaceWorkspace {
    pub workspace: String,
}

#[derive(Debug, Deserialize)]
pub struct AerospaceMonitor {
    #[serde(rename = "monitor-id")]
    pub monitor_id: i32,
    #[serde(rename = "monitor-name")]
    pub monitor_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AerospaceWindow {
    #[serde(rename = "window-id")]
    pub window_id: i32,
    #[serde(rename = "window-title")]
    pub window_title: String,
    #[serde(rename = "app-name")]
    pub app_name: String,
    #[serde(rename = "workspace")]
    pub workspace: String,
}

fn execute_aerospace_command(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("aerospace")
        .arg(command)
        .args(args)
        .output()
        .map_err(|err| format!("Failed to execute aerospace CLI: {}", err))?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 output: {e}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.to_string())
    }
}

enum AerospaceCommand {
    ListApps,
    ListWorkspaces,
    ListMonitors,
    ListWindows,
}

impl AerospaceCommand {
    fn as_str(&self) -> &'static str {
        match self {
            AerospaceCommand::ListApps => "list-apps",
            AerospaceCommand::ListWorkspaces => "list-workspaces",
            AerospaceCommand::ListMonitors => "list-monitors",
            AerospaceCommand::ListWindows => "list-windows",
        }
    }
}

fn query_aerospace<T>(command: AerospaceCommand, args: &[&str]) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let mut json_args = vec!["--json"];
    json_args.extend_from_slice(args);

    let output = execute_aerospace_command(command.as_str(), &json_args)?;

    serde_json::from_str(&output).map_err(|error| {
        format!(
            "Failed to deserialize {} response: {}",
            command.as_str(),
            error
        )
    })
}

fn aerospace_output_format(included_fields: Vec<&str>) -> String {
    format!(
        "{}",
        included_fields
            .iter()
            .map(|field| format!("%{{{}}}", field))
            .collect::<Vec<String>>()
            .join(" ")
    )
}

pub struct Aerospace;

impl Aerospace {
    pub fn ensure_aerospace_installed() {
        if which::which("aerospace").is_err() {
            eprintln!("Error: 'aerospace' command not found. Please install Aerospace CLI tool.");
            std::process::exit(1);
        }
    }

    pub fn list_apps() -> Result<Vec<AerospaceApp>, String> {
        let fields = aerospace_output_format(vec!["app-bundle-id", "app-name", "app-pid"]);
        query_aerospace::<Vec<AerospaceApp>>(AerospaceCommand::ListApps, &["--format", &fields])
    }

    pub fn list_workspaces() -> Result<Vec<AerospaceWorkspace>, String> {
        let fields = aerospace_output_format(vec!["workspace"]);
        query_aerospace(
            AerospaceCommand::ListWorkspaces,
            &["--all", "--format", &fields],
        )
    }

    pub fn list_monitors() -> Result<Vec<AerospaceMonitor>, String> {
        let fields = aerospace_output_format(vec!["monitor-id", "monitor-name"]);
        query_aerospace::<Vec<AerospaceMonitor>>(
            AerospaceCommand::ListMonitors,
            &["--format", &fields],
        )
    }

    pub fn list_windows() -> Result<Vec<AerospaceWindow>, String> {
        let fields =
            aerospace_output_format(vec!["window-id", "window-title", "app-name", "workspace"]);

        query_aerospace::<Vec<AerospaceWindow>>(
            AerospaceCommand::ListWindows,
            &["--all", "--format", &fields],
        )
    }
}
