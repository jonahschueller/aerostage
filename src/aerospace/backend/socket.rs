use std::{
    cell::RefCell,
    env,
    io::{Read, Write},
    marker::PhantomData,
    os::unix::net::UnixStream,
};

use anyhow::Context;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::aerospace::{
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId,
    backend::{
        AerospaceBackend,
        common::{AerospaceCommand, format_aerospace},
        error::{AerospaceBackendError, Result},
    },
};

fn get_aerospace_socket_path() -> Result<String> {
    let env_user = env::var("USER")?;

    Ok(format!("/tmp/bobko.aerospace-{}.sock", env_user))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AerospaceClientRequest {
    args: Vec<String>,
    stdin: String,
    window_id: Option<u32>,
    workspace: Option<u32>,
}

impl AerospaceClientRequest {
    fn new(args: Vec<String>) -> Self {
        Self {
            args,
            stdin: "".to_string(),
            window_id: None,
            workspace: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AerospaceServerResponse {
    exit_code: i32,
    stdout: String,
    stderr: String,
    server_version_and_hash: String,
}

// AerospaceSocketBackend Type State
pub struct ConnectingSocketState;
pub struct OpenSocketState;

pub struct AerospaceSocketBackend<State> {
    socket: RefCell<UnixStream>,
    _state: std::marker::PhantomData<State>,
}

impl AerospaceSocketBackend<ConnectingSocketState> {
    pub fn new(path: &str) -> Result<Self> {
        let socket = UnixStream::connect(path).map_err(|source| {
            AerospaceBackendError::UnavailableAerospace {
                path: path.to_owned(),
                source,
            }
        })?;

        Ok(Self {
            socket: RefCell::new(socket),
            _state: PhantomData,
        })
    }

    pub fn with_aerospace_socket() -> Result<Self> {
        let socket_path = get_aerospace_socket_path()?;

        AerospaceSocketBackend::new(&socket_path)
    }

    pub fn do_handshake(self) -> Result<AerospaceSocketBackend<OpenSocketState>> {
        const SOCKET_PROTOCOL_VERSION: u32 = 1;

        {
            let mut socket = self.socket.borrow_mut();
            socket.write_all(&SOCKET_PROTOCOL_VERSION.to_le_bytes())?;

            let mut server_version_buf = [0u8; 4];
            socket.read_exact(&mut server_version_buf)?;

            let server_version = u32::from_le_bytes(server_version_buf);

            if server_version != SOCKET_PROTOCOL_VERSION {
                return Err(AerospaceBackendError::IncompatibleVersion {
                    client_version: SOCKET_PROTOCOL_VERSION,
                    server_version,
                });
            }
        }

        Ok(AerospaceSocketBackend {
            socket: self.socket,
            _state: PhantomData,
        })
    }
}

impl AerospaceSocketBackend<OpenSocketState> {
    fn write_raw(&self, payload: &[u8]) -> Result<()> {
        let mut socket = self.socket.borrow_mut();
        let len = payload.len() as u32;

        socket.write_all(&len.to_le_bytes())?;
        socket.write_all(payload)?;

        Ok(())
    }

    fn write(&self, payload: &str) -> Result<()> {
        self.write_raw(payload.as_bytes())
    }

    fn write_client_request(&self, payload: &AerospaceClientRequest) -> Result<()> {
        let payload = serde_json::to_string(payload).map_err(AerospaceBackendError::Serialize)?;
        self.write(&payload)
    }

    fn write_command(&self, command: &AerospaceCommand, args: &[&str]) -> Result<()> {
        let full_args: Vec<String> = std::iter::once(command.to_string())
            .chain(args.iter().copied().map(str::to_string))
            .collect();

        let request = AerospaceClientRequest::new(full_args);

        self.write_client_request(&request)
    }

    fn read(&self) -> Result<Vec<u8>> {
        let mut socket = self.socket.borrow_mut();
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf)?;

        let mut payload_buf = vec![0u8; u32::from_le_bytes(len_buf) as usize];
        socket.read_exact(&mut payload_buf)?;

        Ok(payload_buf)
    }

    fn read_str(&self) -> Result<String> {
        let result = self.read()?;

        Ok(String::from_utf8(result)?)
    }

    fn read_response(&self) -> Result<AerospaceServerResponse> {
        let result = self.read_str()?;

        serde_json::from_str(&result).map_err(AerospaceBackendError::Deserialize)
    }

    fn execute_command(
        &self,
        command: &AerospaceCommand,
        args: &[&str],
    ) -> Result<AerospaceServerResponse> {
        self.write_command(command, args)?;
        let response = self.read_response()?;

        if response.exit_code != 0 {
            return Err(AerospaceBackendError::CommandFailed {
                command: command.to_string(),
                exit_code: response.exit_code,
                stderr: response.stderr,
            });
        }

        Ok(response)
    }

    fn query_command<T>(&self, command: &AerospaceCommand, args: &[&str]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let full_args: Vec<&str> = std::iter::once("--json")
            .chain(args.iter().copied())
            .collect();
        let response = self.execute_command(command, &full_args)?;

        serde_json::from_str(&response.stdout).map_err(AerospaceBackendError::Deserialize)
    }
}

impl AerospaceBackend for AerospaceSocketBackend<OpenSocketState> {
    fn list_apps(&self) -> anyhow::Result<Vec<AerospaceApp>> {
        let fields = format_aerospace(&["app-bundle-id", "app-name", "app-pid"]);
        self.query_command::<Vec<AerospaceApp>>(&AerospaceCommand::ListApps, &["--format", &fields])
            .context("Failed to execute list-apps.")
    }

    fn list_workspaces(&self) -> anyhow::Result<Vec<AerospaceWorkspace>> {
        let fields = format_aerospace(&["workspace", "workspace-root-container-layout"]);
        self.query_command(
            &AerospaceCommand::ListWorkspaces,
            &["--all", "--format", &fields],
        )
        .context("Failed to execute list-workspaces.")
    }

    fn list_windows(&self) -> anyhow::Result<Vec<AerospaceWindow>> {
        let fields = format_aerospace(&[
            "window-id",
            "window-title",
            "app-name",
            "app-bundle-id",
            "workspace",
        ]);

        self.query_command::<Vec<AerospaceWindow>>(
            &AerospaceCommand::ListWindows,
            &["--all", "--format", &fields],
        )
        .context("Failed to execute list-windows.")
    }

    fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> anyhow::Result<()> {
        let win_id_arg = format!("{}", window_id);

        self.execute_command(
            &AerospaceCommand::MoveNodeToWorkspace,
            &["--window-id", &win_id_arg, "--", workspace],
        )
        .context("Failed to execute move_node_to_workspace.")?;

        Ok(())
    }

    fn layout(
        &self,
        workspace: &AerospaceWorkspaceId,
        layout: &AerospaceLayout,
    ) -> anyhow::Result<()> {
        let layout_str = layout.to_string();

        self.execute_command(
            &AerospaceCommand::ChangeLayout,
            &["--workspace", workspace, "--root", &layout_str],
        )
        .context("Failed to execute 'layout'.")?;

        Ok(())
    }

    fn flatten_workspace_tree(&self, workspace: &AerospaceWorkspaceId) -> anyhow::Result<()> {
        self.execute_command(
            &AerospaceCommand::FlattenWorkspaceTree,
            &["--workspace", workspace],
        )
        .context("Failed to execute flatten_workspace_tree.")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::aerospace::backend::{
        AerospaceBackend, AerospaceSocketBackend, common::AerospaceCommand,
    };

    #[test]
    fn test_aerospace_handshake() {
        let backend = AerospaceSocketBackend::with_aerospace_socket()
            .expect("Failed to create Aerospace socket backend.");

        let conn_backend = backend.do_handshake().expect("Failed to perform handshake");

        let result = conn_backend
            .list_workspaces()
            .expect("Failed to list workspaces");
        dbg!(&result);
    }
}
