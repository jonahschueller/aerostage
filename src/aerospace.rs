use serde::de::DeserializeOwned;
use std::{fmt::Display, process::Command};

use serde::Deserialize;

use anyhow::{Context, Result, anyhow, ensure};

pub type AerospaceWindowId = u32;
pub type AerospaceWorkspaceId = String;

#[derive(Debug, Deserialize)]
pub struct AerospaceApp {
    #[serde(rename = "app-bundle-id")]
    pub app_bundle_id: String,
    #[serde(rename = "app-name")]
    pub app_name: String,
    #[serde(rename = "app-pid")]
    pub app_pid: u32,
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
    fn execute(&self, command: &str, args: &[&str]) -> Result<String>;
}

pub struct AerospaceCommandExecutor;

impl CommandExecutor for AerospaceCommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new("aerospace")
            .arg(command)
            .args(args)
            .output()
            .map_err(|err| anyhow!("Failed to execute aerospace CLI: {}", err))?;

        ensure!(
            output.status.success(),
            "Aerospace command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8(output.stdout).map_err(|e| anyhow!("Invalid UTF-8 output: {e}"))
    }
}

#[cfg(test)]
pub struct MockExecutor {
    pub stdout: String,
    pub should_succeed: bool,
}

#[cfg(test)]
impl Default for MockExecutor {
    fn default() -> Self {
        Self {
            stdout: "[]".to_string(),
            should_succeed: true,
        }
    }
}

#[cfg(test)]
impl CommandExecutor for MockExecutor {
    fn execute(&self, _command: &str, _args: &[&str]) -> Result<String> {
        if self.should_succeed {
            Ok(self.stdout.clone())
        } else {
            Err(anyhow::anyhow!("Mocked CLI failure"))
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

impl Display for AerospaceCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AerospaceCommand::ListApps => "list-apps",
            AerospaceCommand::ListWorkspaces => "list-workspaces",
            AerospaceCommand::ListMonitors => "list-monitors",
            AerospaceCommand::ListWindows => "list-windows",
            AerospaceCommand::MoveNodeToWorkspace => "move-node-to-workspace",
        };
        write!(f, "{s}")
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
            panic!("Error: 'aerospace' command not found. Please install Aerospace CLI tool.");
        }
    }
}

#[cfg(test)]
impl Aerospace<MockExecutor> {
    #[cfg(test)]
    fn mock() -> Self {
        Self {
            executor: MockExecutor::default(),
        }
    }
}

impl<E: CommandExecutor> Aerospace<E> {
    #[allow(dead_code)]
    pub fn new(executor: E) -> Self {
        Self { executor }
    }

    fn execute_aerospace(&self, command: &AerospaceCommand, args: &[&str]) -> Result<String> {
        self.executor.execute(&format!("{}", command), &args)
    }

    fn query_aerospace<T>(&self, command: &AerospaceCommand, args: &[&str]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let full_args: Vec<&str> = std::iter::once("--json")
            .chain(args.iter().copied())
            .collect();

        let output = self.execute_aerospace(command, &full_args)?;

        serde_json::from_str(&output)
            .map_err(|error| anyhow!("Failed to deserialize {} response: {}", command, error))
    }

    fn aerospace_output_format(&self, included_fields: &[&str]) -> String {
        included_fields
            .iter()
            .map(|field| format!("%{{{}}}", field))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[allow(dead_code)]
    pub fn list_apps(&self) -> Result<Vec<AerospaceApp>> {
        let fields = self.aerospace_output_format(&["app-bundle-id", "app-name", "app-pid"]);
        self.query_aerospace::<Vec<AerospaceApp>>(
            &AerospaceCommand::ListApps,
            &["--format", &fields],
        )
        .with_context(|| "Failed to execute list-apps.")
    }

    pub fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>> {
        let fields = self.aerospace_output_format(&["workspace"]);
        self.query_aerospace(
            &AerospaceCommand::ListWorkspaces,
            &["--all", "--format", &fields],
        )
        .with_context(|| "Failed to execute list-workspaces.")
    }

    #[allow(dead_code)]
    pub fn list_monitors(&self) -> Result<Vec<AerospaceMonitor>> {
        let fields = self.aerospace_output_format(&["monitor-id", "monitor-name"]);
        self.query_aerospace::<Vec<AerospaceMonitor>>(
            &AerospaceCommand::ListMonitors,
            &["--format", &fields],
        )
        .with_context(|| "Failed to execute list-monitors.")
    }

    pub fn list_windows(&self) -> Result<Vec<AerospaceWindow>> {
        let fields = self.aerospace_output_format(&[
            "window-id",
            "window-title",
            "app-name",
            "app-bundle-id",
            "workspace",
        ]);

        self.query_aerospace::<Vec<AerospaceWindow>>(
            &AerospaceCommand::ListWindows,
            &["--all", "--format", &fields],
        )
        .with_context(|| "Failed to execute list-windows.")
    }

    pub fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        let win_id_arg = format!("{}", window_id);

        self.execute_aerospace(
            &AerospaceCommand::MoveNodeToWorkspace,
            &["--window-id", &win_id_arg, workspace],
        )
        .with_context(|| "Failed to execute move_node_to_workspace.")?;

        Ok(())
    }
}

impl AerospaceWindow {
    #[cfg(test)]
    pub fn dummy() -> Self {
        AerospaceWindow {
            app_bundle_id: "com.example.test".into(),
            app_name: "Test App".into(),
            window_id: 0,
            workspace: "1".into(),
            window_title: "Test Title".into(),
        }
    }

    #[cfg(test)]
    pub fn with_bundle_id(mut self, bundle_id: &str) -> Self {
        self.app_bundle_id = bundle_id.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_window_id(mut self, window_id: u32) -> Self {
        self.window_id = window_id;
        self
    }

    #[cfg(test)]
    pub fn with_app_name(mut self, app_name: &str) -> Self {
        self.app_name = app_name.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_workspace(mut self, workspace: &str) -> Self {
        self.workspace = workspace.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_window_title(mut self, window_title: &str) -> Self {
        self.window_title = window_title.to_string();
        self
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    pub struct MockAerospaceCommandExecutor {
        result: Result<String>,
    }

    impl MockAerospaceCommandExecutor {
        fn with_success(value: &str) -> MockAerospaceCommandExecutor {
            MockAerospaceCommandExecutor {
                result: Ok(value.to_string()),
            }
        }

        fn with_failure(error: &str) -> MockAerospaceCommandExecutor {
            MockAerospaceCommandExecutor {
                result: Err(anyhow!(error.to_string())),
            }
        }
    }

    impl CommandExecutor for MockAerospaceCommandExecutor {
        fn execute(&self, command: &str, args: &[&str]) -> Result<String> {
            match &self.result {
                Ok(val) => Ok(val.clone()),
                Err(err) => Err(anyhow::anyhow!("{err}")),
            }
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
                "app-bundle-id": "com.example.test",
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
        assert_eq!(test_window.app_bundle_id, "com.example.test");
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
