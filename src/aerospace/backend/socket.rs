use std::{
    env,
    io::{Read, Write},
    marker::PhantomData,
    os::unix::net::UnixStream,
};

use anyhow::{Ok, Result, ensure};

use crate::aerospace::{
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId, backend::AerospaceBackend,
};

fn get_aerospace_socket_path() -> Result<String> {
    let env_user = env::var("USER")?;

    Ok(format!("/tmp/bobko.aerospace-{}.sock", env_user))
}

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
    use crate::aerospace::backend::AerospaceSocketBackend;

    #[test]
    fn test_aerospace_handshake() {
        let backend = AerospaceSocketBackend::with_aerospace_socket()
            .expect("Failed to create Aerospace socket backend.");

        let mut conn_backend = backend.do_handshake().expect("Failed to perform handshake");
    }
}
