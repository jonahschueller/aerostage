use serde::de::DeserializeOwned;
use subprocess::{Exec, Redirection};

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

fn execute_aerospace_command(command: &str) -> Result<String, String> {
    const AEROSPACE_COMMAND: &str = "aerospace";

    let args: Vec<&str> = command.split_whitespace().collect();

    let output = Exec::cmd(AEROSPACE_COMMAND)
        .args(&args)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .capture()
        .expect("Failed to execute aerospace command");

    if output.success() {
        Ok(output.stdout_str().to_string())
    } else {
        Err(output.stderr_str().to_string())
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

fn query_aerospace<T>(command: AerospaceCommand, args: Option<&str>) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let json_query = match args {
        Some(a) => format!("{} --json {}", command.as_str(), a),
        None => format!("{} --json", command.as_str()),
    };

    let output = execute_aerospace_command(&json_query)?;

    serde_json::from_str(&output).map_err(|error| {
        format!(
            "Failed to deserialize {} response: {}",
            command.as_str(),
            error
        )
    })
}

fn aerospace_output_format(included_fields: Vec<&str>) -> String {
    included_fields
        .iter()
        .map(|field| format!("%{{{}}}", field))
        .collect::<Vec<String>>()
        .join(",")
}

fn aerospace_args(args: Vec<&str>) -> String {
    args.join(" ")
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
        let args = aerospace_args(vec![&fields]);
        query_aerospace::<Vec<AerospaceApp>>(AerospaceCommand::ListApps, Some(&args))
    }

    pub fn list_workspaces() -> Result<Vec<AerospaceWorkspace>, String> {
        let fields = aerospace_output_format(vec!["workspace"]);
        let args = aerospace_args(vec!["--all", &fields]);
        query_aerospace(AerospaceCommand::ListWorkspaces, Some(&args))
    }

    pub fn list_monitors() -> Result<Vec<AerospaceMonitor>, String> {
        let fields = aerospace_output_format(vec!["monitor-id", "monitor-name"]);
        let args = aerospace_args(vec![&fields]);
        query_aerospace::<Vec<AerospaceMonitor>>(AerospaceCommand::ListMonitors, Some(&args))
    }

    pub fn list_windows() -> Result<Vec<AerospaceWindow>, String> {
        let fields =
            aerospace_output_format(vec!["window-id", "window-title", "app-name", "workspace"]);
        let args = aerospace_args(vec!["--all", &fields]);
        query_aerospace::<Vec<AerospaceWindow>>(AerospaceCommand::ListWindows, Some(&args))
    }
}
