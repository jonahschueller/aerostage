use std::fmt::Display;
use std::process::Command;

use anyhow::{Context, Result, anyhow, ensure};
use serde::de::DeserializeOwned;

use crate::aerospace::{
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId, backend::AerospaceBackend,
};

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

pub enum AerospaceCommand {
    ListApps,
    ListWorkspaces,
    ListMonitors,
    ListWindows,
    MoveNodeToWorkspace,
    ChangeLayout,
    FlattenWorkspaceTree,
}

impl Display for AerospaceCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AerospaceCommand::ListApps => "list-apps",
            AerospaceCommand::ListWorkspaces => "list-workspaces",
            AerospaceCommand::ListMonitors => "list-monitors",
            AerospaceCommand::ListWindows => "list-windows",
            AerospaceCommand::MoveNodeToWorkspace => "move-node-to-workspace",
            AerospaceCommand::ChangeLayout => "layout",
            AerospaceCommand::FlattenWorkspaceTree => "flatten-workspace-tree",
        };
        write!(f, "{s}")
    }
}

pub struct AerospaceCliCBackend<E: CommandExecutor = AerospaceCommandExecutor> {
    executor: E,
}

impl Default for AerospaceCliCBackend {
    fn default() -> Self {
        Self {
            executor: AerospaceCommandExecutor {},
        }
    }
}

impl<E: CommandExecutor> AerospaceCliCBackend<E> {
    pub fn new(executor: E) -> Self {
        Self { executor }
    }

    fn execute_aerospace(&self, command: &AerospaceCommand, args: &[&str]) -> Result<String> {
        self.executor.execute(&format!("{}", command), args)
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
}

impl<E: CommandExecutor> AerospaceBackend for AerospaceCliCBackend<E> {
    fn list_apps(&self) -> Result<Vec<AerospaceApp>> {
        let fields = self.aerospace_output_format(&["app-bundle-id", "app-name", "app-pid"]);
        self.query_aerospace::<Vec<AerospaceApp>>(
            &AerospaceCommand::ListApps,
            &["--format", &fields],
        )
        .with_context(|| "Failed to execute list-apps.")
    }

    fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>> {
        let fields =
            self.aerospace_output_format(&["workspace", "workspace-root-container-layout"]);
        self.query_aerospace(
            &AerospaceCommand::ListWorkspaces,
            &["--all", "--format", &fields],
        )
        .with_context(|| "Failed to execute list-workspaces.")
    }

    // pub fn list_monitors(&self) -> Result<Vec<AerospaceMonitor>> {
    //     let fields = self.aerospace_output_format(&["monitor-id", "monitor-name"]);
    //     self.query_aerospace::<Vec<AerospaceMonitor>>(
    //         &AerospaceCommand::ListMonitors,
    //         &["--format", &fields],
    //     )
    //     .with_context(|| "Failed to execute list-monitors.")
    // }

    fn list_windows(&self) -> Result<Vec<AerospaceWindow>> {
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

    fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        let win_id_arg = format!("{}", window_id);

        self.execute_aerospace(
            &AerospaceCommand::MoveNodeToWorkspace,
            &["--window-id", &win_id_arg, "--", workspace],
        )
        .with_context(|| "Failed to execute move_node_to_workspace.")?;

        Ok(())
    }

    fn layout(&self, workspace: &AerospaceWorkspaceId, layout: &AerospaceLayout) -> Result<()> {
        let layout_str = layout.to_string();

        self.execute_aerospace(
            &AerospaceCommand::ChangeLayout,
            &["--workspace", workspace, "--root", &layout_str],
        )
        .with_context(|| "Failed to execute 'layout'.")?;

        Ok(())
    }

    fn flatten_workspace_tree(&self, workspace: &AerospaceWorkspaceId) -> Result<()> {
        self.execute_aerospace(
            &AerospaceCommand::FlattenWorkspaceTree,
            &["--workspace", workspace],
        )
        .with_context(|| "Failed to execute flatten_workspace_tree.")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAerospaceCommandExecutor {
        stdout: String,
        should_succeed: bool,
    }

    impl MockAerospaceCommandExecutor {
        fn with_success(stdout: &str) -> Self {
            Self {
                stdout: stdout.to_string(),
                should_succeed: true,
            }
        }

        fn with_failure(_message: &str) -> Self {
            Self {
                stdout: String::new(),
                should_succeed: false,
            }
        }
    }

    impl CommandExecutor for MockAerospaceCommandExecutor {
        fn execute(&self, _command: &str, _args: &[&str]) -> Result<String> {
            if self.should_succeed {
                Ok(self.stdout.clone())
            } else {
                Err(anyhow!("Mocked CLI failure"))
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

        let backend = AerospaceCliCBackend::new(executor);

        let apps = backend.list_apps().expect("Should parse listed apps.");

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

        let backend = AerospaceCliCBackend::new(executor);

        let apps = backend.list_apps();

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

        let backend = AerospaceCliCBackend::new(executor);

        let windows = backend
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

        let backend = AerospaceCliCBackend::new(executor);

        let windows = backend.list_windows();

        assert!(windows.is_err());
    }

    #[test]
    fn test_list_windows_missing_optional_fields_default() {
        let executor = MockAerospaceCommandExecutor::with_success(
            r#"[{
                "window-id" : 1,
                "app-name" : "TestApp",
                "workspace" : "1"
            }]"#,
        );

        let backend = AerospaceCliCBackend::new(executor);
        let windows = backend
            .list_windows()
            .expect("Should parse listed windows.");
        let window = windows.first().unwrap();

        assert_eq!(window.window_id, 1);
        assert_eq!(window.window_title, "");
        assert_eq!(window.app_bundle_id, "");
    }
}
