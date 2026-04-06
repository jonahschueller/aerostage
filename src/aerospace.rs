use serde::{Deserialize, de::DeserializeOwned};
use subprocess::{Exec, Redirection};

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

pub fn ensure_aerospace_installed() {
    if which::which("aerospace").is_err() {
        eprintln!("Error: 'aerospace' command not found. Please install Aerospace CLI tool.");
        std::process::exit(1);
    }
}

#[derive(Debug, Deserialize)]
struct AerospaceApp {
    #[serde(rename = "app-bundle-id")]
    app_bundle_id: String,
    #[serde(rename = "app-name")]
    app_name: String,
    #[serde(rename = "app-pid")]
    app_pid: i32,
}

pub fn aerospace_list_apps() {
    let result = query_aerospace::<Vec<AerospaceApp>>(AerospaceCommand::ListApps, None);
    match result {
        Ok(output) => println!("{:#?}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}

#[derive(Debug, Deserialize)]
struct AerospaceWorkspace {
    workspace: String,
}

pub fn aerospace_list_workspaces() {
    let result =
        query_aerospace::<Vec<AerospaceWorkspace>>(AerospaceCommand::ListWorkspaces, Some("--all"));
    match result {
        Ok(output) => println!("{:#?}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}

#[derive(Debug, Deserialize)]
struct AerospaceMonitor {
    #[serde(rename = "monitor-id")]
    monitor_id: i32,
    #[serde(rename = "monitor-name")]
    monitor_name: String,
}

pub fn aerospace_list_monitors() {
    let result = query_aerospace::<Vec<AerospaceMonitor>>(AerospaceCommand::ListMonitors, None);
    match result {
        Ok(output) => println!("{:#?}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}

#[derive(Debug, Deserialize)]
struct AerospaceWindow {
    #[serde(rename = "window-id")]
    window_id: i32,
    #[serde(rename = "window-title")]
    window_title: String,
    #[serde(rename = "app-name")]
    app_name: String,
}

pub fn aerospace_list_windows() {
    let result =
        query_aerospace::<Vec<AerospaceWindow>>(AerospaceCommand::ListWindows, Some("--all"));
    match result {
        Ok(output) => println!("{:#?}", output),
        Err(error) => eprintln!("Error: {}", error),
    }
}
