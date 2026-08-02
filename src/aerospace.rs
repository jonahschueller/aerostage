use serde::de::DeserializeOwned;
use std::{default, fmt::format, process::Command};

use serde::Deserialize;

pub type AerospaceWindowId = i32;
pub type AerospaceWorkspaceId = String;

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
    pub workspace: AerospaceWorkspaceId,
}

#[derive(Debug, Deserialize)]
pub struct AerospaceMonitor {
    #[serde(rename = "monitor-id")]
    pub monitor_id: i32,
    #[serde(rename = "monitor-name")]
    pub monitor_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AerospaceWindow {
    #[serde(rename = "window-id")]
    pub window_id: AerospaceWindowId,
    #[serde(rename = "window-title")]
    pub window_title: String,
    #[serde(rename = "app-name")]
    pub app_name: String,
    #[serde(rename = "app-bundle-id")]
    pub app_bundle_id: String,
    #[serde(rename = "workspace")]
    pub workspace: AerospaceWorkspaceId,
}

pub trait CommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<String, String>;
}

pub struct AerospaceCommandExecutor;

impl CommandExecutor for AerospaceCommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<String, String> {
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
}

enum AerospaceCommand {
    ListApps,
    ListWorkspaces,
    ListMonitors,
    ListWindows,
    MoveNodeToWorkspace,
}

impl AerospaceCommand {
    fn as_str(&self) -> &'static str {
        match self {
            AerospaceCommand::ListApps => "list-apps",
            AerospaceCommand::ListWorkspaces => "list-workspaces",
            AerospaceCommand::ListMonitors => "list-monitors",
            AerospaceCommand::ListWindows => "list-windows",
            AerospaceCommand::MoveNodeToWorkspace => "move-node-to-workspace",
        }
    }
}

pub struct Aerospace<E: CommandExecutor = AerospaceCommandExecutor> {
    executor: E,
}

impl Default for Aerospace {
    fn default() -> Self {
        Self {
            executor: AerospaceCommandExecutor {},
        }
    }
}

impl Aerospace {
    pub fn ensure_aerospace_installed() {
        if which::which("aerospace").is_err() {
            eprintln!("Error: 'aerospace' command not found. Please install Aerospace CLI tool.");
            std::process::exit(1);
        }
    }
}

impl<E: CommandExecutor> Aerospace<E> {
    pub fn new(executor: E) -> Self {
        Self { executor }
    }

    fn query_aerospace<T>(&self, command: AerospaceCommand, args: &[&str]) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let mut json_args = vec!["--json"];
        json_args.extend_from_slice(args);

        let output = self.executor.execute(command.as_str(), &json_args)?;

        serde_json::from_str(&output).map_err(|error| {
            format!(
                "Failed to deserialize {} response: {}",
                command.as_str(),
                error
            )
        })
    }

    fn aerospace_output_format(&self, included_fields: Vec<&str>) -> String {
        format!(
            "{}",
            included_fields
                .iter()
                .map(|field| format!("%{{{}}}", field))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    pub fn list_apps(&self) -> Result<Vec<AerospaceApp>, String> {
        let fields = self.aerospace_output_format(vec!["app-bundle-id", "app-name", "app-pid"]);
        self.query_aerospace::<Vec<AerospaceApp>>(
            AerospaceCommand::ListApps,
            &["--format", &fields],
        )
    }

    pub fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>, String> {
        let fields = self.aerospace_output_format(vec!["workspace"]);
        self.query_aerospace(
            AerospaceCommand::ListWorkspaces,
            &["--all", "--format", &fields],
        )
    }

    pub fn list_monitors(&self) -> Result<Vec<AerospaceMonitor>, String> {
        let fields = self.aerospace_output_format(vec!["monitor-id", "monitor-name"]);
        self.query_aerospace::<Vec<AerospaceMonitor>>(
            AerospaceCommand::ListMonitors,
            &["--format", &fields],
        )
    }

    pub fn list_windows(&self) -> Result<Vec<AerospaceWindow>, String> {
        let fields = self.aerospace_output_format(vec![
            "window-id",
            "window-title",
            "app-name",
            "app-bundle-id",
            "workspace",
        ]);

        self.query_aerospace::<Vec<AerospaceWindow>>(
            AerospaceCommand::ListWindows,
            &["--all", "--format", &fields],
        )
    }

    pub fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: &AerospaceWindowId,
    ) -> Result<(), String> {
        let win_id_arg = format!("{}", window_id);

        match self.executor.execute(
            AerospaceCommand::MoveNodeToWorkspace.as_str(),
            &["--window-id", &win_id_arg, workspace],
        ) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct MockAerospaceCommandExecutor {
        result: Result<String, String>,
    }

    impl MockAerospaceCommandExecutor {
        fn with_success(value: &str) -> MockAerospaceCommandExecutor {
            MockAerospaceCommandExecutor {
                result: Ok(value.to_string()),
            }
        }

        fn with_failure(error: &str) -> MockAerospaceCommandExecutor {
            MockAerospaceCommandExecutor {
                result: Err(error.to_string()),
            }
        }
    }

    impl CommandExecutor for MockAerospaceCommandExecutor {
        fn execute(&self, command: &str, args: &[&str]) -> Result<String, String> {
            return self.result.clone();
        }
    }

    #[test]
    fn test_lists_apps_successfully() {
        let executor = MockAerospaceCommandExecutor::with_success(
            r#"[{
                "app-name" : "TestApp",
                "app-bundle-id" : "com.test.app",
                "app-pid" : 42
            }
            ]"#,
        );

        let aerospace = Aerospace::new(executor);

        let apps = aerospace.list_apps().expect("Should parse listed apps.");

        assert_eq!(apps.len(), 1);

        let test_app = apps.first().expect("Should have first app.");
        assert_eq!(test_app.app_name, "TestApp");
        assert_eq!(test_app.app_bundle_id, "com.test.app");
        assert_eq!(test_app.app_pid, 42);
    }

    #[test]
    fn test_lists_apps_failure() {
        let executor =
            MockAerospaceCommandExecutor::with_failure(r#"Failed to execute aerospace."#);

        let aerospace = Aerospace::new(executor);

        let apps = aerospace.list_apps();

        assert!(apps.is_err());
    }

    #[test]
    fn test_list_windows_successfully() {
        let executor = MockAerospaceCommandExecutor::with_success(
            r#"[{
                "window-id" : 1,
                "window-title" : "TestWindow",
                "app-name" : "TestApp",
                "workspace" : "TestWorkspace"
            }
            ]"#,
        );

        let aerospace = Aerospace::new(executor);

        let windows = aerospace
            .list_windows()
            .expect("Should parse listed windows.");

        assert_eq!(windows.len(), 1);

        let test_window = windows.first().expect("Should have first window.");
        assert_eq!(test_window.window_id, 1);
        assert_eq!(test_window.window_title, "TestWindow");
        assert_eq!(test_window.app_name, "TestApp");
        assert_eq!(test_window.workspace, "TestWorkspace");
    }

    #[test]
    fn test_list_windows_failure() {
        let executor = MockAerospaceCommandExecutor::with_failure(
            r#"ERROR: Failed to parse <output-format>. Unbalanced curly braces"#,
        );

        let aerospace = Aerospace::new(executor);

        let windows = aerospace.list_windows();

        assert!(windows.is_err());
    }
}
