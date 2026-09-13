use std::{
    env,
    io::{Read, Write},
    marker::PhantomData,
    os::unix::net::UnixStream,
};

use anyhow::{Ok, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::aerospace::{
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId,
    backend::{AerospaceBackend, common::AerospaceCommand},
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

pub struct AerospaceSocketBackend<State = OpenSocketState> {
    socket: UnixStream,
    _state: std::marker::PhantomData<State>,
}

impl AerospaceSocketBackend<ConnectingSocketState> {
    pub fn new(path: &str) -> Result<Self> {
        let socket = UnixStream::connect(path)?;

        Ok(Self {
            socket,
            _state: PhantomData,
        })
    }

    pub fn with_aerospace_socket() -> Result<Self> {
        let socket_path = get_aerospace_socket_path()?;

        AerospaceSocketBackend::new(&socket_path)
    }

    pub fn do_handshake(mut self) -> Result<AerospaceSocketBackend<OpenSocketState>> {
        const SOCKET_PROTOCOL_VERSION: u32 = 1;

        self.socket
            .write_all(&SOCKET_PROTOCOL_VERSION.to_le_bytes())?;

        let mut server_version_buf = [0u8; 4];
        self.socket.read_exact(&mut server_version_buf)?;

        let server_version = u32::from_le_bytes(server_version_buf);

        ensure!(
            server_version == SOCKET_PROTOCOL_VERSION,
            "Aerospace server returned different protocol version (client: {}, server: {})",
            &SOCKET_PROTOCOL_VERSION,
            &server_version
        );

        Ok(AerospaceSocketBackend {
            socket: self.socket,
            _state: PhantomData,
        })
    }
}

impl AerospaceSocketBackend<OpenSocketState> {
    fn write_raw(&mut self, payload: &[u8]) -> Result<()> {
        let len = payload.len() as u32;

        let len_buf = len.to_le_bytes();
        self.socket.write_all(&len_buf)?;
        self.socket.write_all(payload)?;

        Ok(())
    }

    fn write(&mut self, payload: &str) -> Result<()> {
        self.write_raw(payload.as_bytes())
    }

    fn write_client_request(&mut self, payload: &AerospaceClientRequest) -> Result<()> {
        let payload = serde_json::to_string(payload)?;
        self.write(&payload)
    }

    fn write_command(&mut self, command: &AerospaceCommand, args: &[&str]) -> Result<()> {
        let full_args: Vec<String> = std::iter::once(command.to_string())
            .chain(args.iter().copied().map(str::to_string))
            .collect();

        let request = AerospaceClientRequest::new(full_args);

        self.write_client_request(&request)
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        self.socket.read_exact(&mut len_buf)?;

        let mut payload_buf = vec![0u8; u32::from_le_bytes(len_buf) as usize];
        self.socket.read_exact(&mut payload_buf)?;

        Ok(payload_buf)
    }

    fn read_str(&mut self) -> Result<String> {
        let result = self.read()?;

        Ok(String::from_utf8(result)?)
    }

    fn read_response(&mut self) -> Result<AerospaceServerResponse> {
        let result = self.read_str()?;

        Ok(serde_json::from_str(&result)?)
    }
}

impl AerospaceBackend for AerospaceSocketBackend<OpenSocketState> {
    fn list_apps(&self) -> Result<Vec<AerospaceApp>> {
        todo!("Not implemented");
    }

    fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>> {
        todo!("Not implemented");
    }

    fn list_windows(&self) -> Result<Vec<AerospaceWindow>> {
        todo!("Not implemented");
    }

    fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        todo!("Not implemented");
    }

    fn layout(&self, workspace: &AerospaceWorkspaceId, layout: &AerospaceLayout) -> Result<()> {
        todo!("Not implemented");
    }

    fn flatten_workspace_tree(&self, workspace: &AerospaceWorkspaceId) -> Result<()> {
        todo!("Not implemented");
    }
}

#[cfg(test)]
mod tests {
    use crate::aerospace::backend::{AerospaceSocketBackend, common::AerospaceCommand};

    #[test]
    fn test_aerospace_handshake() {
        let backend = AerospaceSocketBackend::with_aerospace_socket()
            .expect("Failed to create Aerospace socket backend.");

        let mut conn_backend = backend.do_handshake().expect("Failed to perform handshake");

        let result =
            conn_backend.write_command(&AerospaceCommand::ListWorkspaces, &["--all", "--json"]);

        let response = conn_backend
            .read_response()
            .expect("Failed to obtain response");

        dbg!(&response);
    }
}
