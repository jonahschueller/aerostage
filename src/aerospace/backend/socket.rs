use std::{
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
    workspace: Option<String>,
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
    socket: UnixStream,
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
            socket,
            _state: PhantomData,
        })
    }

    pub fn with_aerospace_socket() -> Result<Self> {
        let socket_path = get_aerospace_socket_path()?;

        AerospaceSocketBackend::new(&socket_path)
    }

    #[cfg(test)]
    fn from_stream(socket: UnixStream) -> Self {
        Self {
            socket,
            _state: PhantomData,
        }
    }

    pub fn do_handshake(self) -> Result<AerospaceSocketBackend<OpenSocketState>> {
        const SOCKET_PROTOCOL_VERSION: u32 = 1;

        {
            let mut socket = &self.socket;
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
        let mut socket = &self.socket;
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
        let mut socket = &self.socket;
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
    use super::*;
    use crate::aerospace::{AerospaceLayout, backend::AerospaceBackend};
    use serde_json::json;
    use std::{
        io::{Read, Write},
        os::unix::net::UnixStream,
        sync::mpsc,
        thread::{self, JoinHandle},
    };

    const PROTOCOL_VERSION: u32 = 1;

    struct FakeAerospacePeer {
        stream: UnixStream,
    }

    impl FakeAerospacePeer {
        fn handshake(&mut self, server_version: u32) {
            let mut client_version = [0u8; 4];
            self.stream
                .read_exact(&mut client_version)
                .expect("peer should read handshake version");
            assert_eq!(u32::from_le_bytes(client_version), PROTOCOL_VERSION);

            self.stream
                .write_all(&server_version.to_le_bytes())
                .expect("peer should write handshake version");
        }

        fn read_request(&mut self) -> serde_json::Value {
            let mut len_buf = [0u8; 4];
            self.stream
                .read_exact(&mut len_buf)
                .expect("peer should read request length");

            let mut payload = vec![0u8; u32::from_le_bytes(len_buf) as usize];
            self.stream
                .read_exact(&mut payload)
                .expect("peer should read request payload");

            serde_json::from_slice(&payload).expect("request should be JSON")
        }

        fn write_payload(&mut self, payload: &[u8]) {
            let len = payload.len() as u32;
            self.stream
                .write_all(&len.to_le_bytes())
                .expect("peer should write payload length");
            self.stream
                .write_all(payload)
                .expect("peer should write payload");
        }

        fn write_response(&mut self, body: serde_json::Value) {
            self.write_payload(body.to_string().as_bytes());
        }

        fn write_success(&mut self, stdout: &str) {
            self.write_response(json!({
                "exitCode": 0,
                "stdout": stdout,
                "stderr": "",
                "serverVersionAndHash": "test-0"
            }));
        }

        fn write_failure(&mut self, exit_code: i32, stderr: &str) {
            self.write_response(json!({
                "exitCode": exit_code,
                "stdout": "",
                "stderr": stderr,
                "serverVersionAndHash": "test-0"
            }));
        }
    }

    fn spawn_peer(
        handler: impl FnOnce(&mut FakeAerospacePeer) + Send + 'static,
    ) -> (
        AerospaceSocketBackend<ConnectingSocketState>,
        JoinHandle<()>,
    ) {
        let (client, server) =
            UnixStream::pair().expect("should create connected unix stream pair");
        let handle = thread::spawn(move || {
            let mut peer = FakeAerospacePeer { stream: server };
            handler(&mut peer);
        });

        (AerospaceSocketBackend::from_stream(client), handle)
    }

    fn connect_handshaken(
        handler: impl FnOnce(&mut FakeAerospacePeer) + Send + 'static,
    ) -> (AerospaceSocketBackend<OpenSocketState>, JoinHandle<()>) {
        let (backend, handle) = spawn_peer(move |peer| {
            peer.handshake(PROTOCOL_VERSION);
            handler(peer);
        });
        let backend = backend
            .do_handshake()
            .expect("handshake with matching version should succeed");
        (backend, handle)
    }

    fn request_args(request: &serde_json::Value) -> Vec<String> {
        request["args"]
            .as_array()
            .expect("request should include args")
            .iter()
            .map(|value| value.as_str().expect("arg should be a string").to_string())
            .collect()
    }

    #[test]
    fn client_request_serializes_workspace_as_string_or_null() {
        let request = AerospaceClientRequest::new(vec!["list-workspaces".into()]);
        let json = serde_json::to_value(&request).expect("request should serialize");

        assert_eq!(json["workspace"], serde_json::Value::Null);
        assert_eq!(json["windowId"], serde_json::Value::Null);

        let request = AerospaceClientRequest {
            args: vec!["layout".into()],
            stdin: String::new(),
            window_id: Some(42),
            workspace: Some("main".into()),
        };
        let json = serde_json::to_value(&request).expect("request should serialize");

        assert_eq!(json["workspace"], "main");
        assert_eq!(json["windowId"], 42);
    }

    #[test]
    fn new_reports_unavailable_aerospace_for_missing_socket() {
        let Err(error) = AerospaceSocketBackend::new("/tmp/aerostage-missing-aerospace.sock")
        else {
            panic!("connecting to a missing socket should fail");
        };

        match error {
            AerospaceBackendError::UnavailableAerospace { path, .. } => {
                assert_eq!(path, "/tmp/aerostage-missing-aerospace.sock");
            }
            other => panic!("expected UnavailableAerospace, got {other:?}"),
        }
    }

    #[test]
    fn handshake_succeeds_when_protocol_versions_match() {
        let (backend, handle) = spawn_peer(|peer| peer.handshake(PROTOCOL_VERSION));

        backend
            .do_handshake()
            .expect("matching protocol versions should handshake");
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn handshake_fails_on_incompatible_server_version() {
        let (backend, handle) = spawn_peer(|peer| peer.handshake(99));

        let Err(error) = backend.do_handshake() else {
            panic!("mismatched protocol versions should fail");
        };
        match error {
            AerospaceBackendError::IncompatibleVersion {
                client_version,
                server_version,
            } => {
                assert_eq!(client_version, PROTOCOL_VERSION);
                assert_eq!(server_version, 99);
            }
            other => panic!("expected IncompatibleVersion, got {other:?}"),
        }
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn list_apps_sends_json_query_and_parses_stdout() {
        let (tx, rx) = mpsc::channel();
        let (backend, handle) = connect_handshaken(move |peer| {
            tx.send(peer.read_request()).unwrap();
            peer.write_success(
                r#"[{
                    "app-name": "TestApp",
                    "app-bundle-id": "com.test.app",
                    "app-pid": 42
                }]"#,
            );
        });

        let apps = backend.list_apps().expect("should parse listed apps");
        let request = rx.recv().expect("peer should receive a request");

        assert_eq!(
            request_args(&request),
            [
                "list-apps",
                "--json",
                "--format",
                "%{app-bundle-id} %{app-name} %{app-pid}"
            ]
        );
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].app_name, "TestApp");
        assert_eq!(apps[0].app_bundle_id, "com.test.app");
        assert_eq!(apps[0].app_pid, 42);
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn list_workspaces_sends_all_and_format_flags() {
        let (tx, rx) = mpsc::channel();
        let (backend, handle) = connect_handshaken(move |peer| {
            tx.send(peer.read_request()).unwrap();
            peer.write_success(
                r#"[{
                    "workspace": "1",
                    "workspace-root-container-layout": "h_tiles"
                }]"#,
            );
        });

        let workspaces = backend
            .list_workspaces()
            .expect("should parse listed workspaces");
        let request = rx.recv().expect("peer should receive a request");

        assert_eq!(
            request_args(&request),
            [
                "list-workspaces",
                "--json",
                "--all",
                "--format",
                "%{workspace} %{workspace-root-container-layout}"
            ]
        );
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].workspace, "1");
        assert_eq!(workspaces[0].layout, AerospaceLayout::HTiles);
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn list_windows_parses_optional_fields() {
        let (backend, handle) = connect_handshaken(|peer| {
            let _ = peer.read_request();
            peer.write_success(
                r#"[{
                    "window-id": 7,
                    "window-title": "TestWindow",
                    "app-name": "TestApp",
                    "app-bundle-id": "com.example.test",
                    "workspace": "main"
                }]"#,
            );
        });

        let windows = backend.list_windows().expect("should parse listed windows");
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].window_id, 7);
        assert_eq!(windows[0].window_title, "TestWindow");
        assert_eq!(windows[0].app_name, "TestApp");
        assert_eq!(windows[0].app_bundle_id, "com.example.test");
        assert_eq!(windows[0].workspace, "main");
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn execute_command_maps_nonzero_exit_to_command_failed() {
        let (backend, handle) = connect_handshaken(|peer| {
            let _ = peer.read_request();
            peer.write_failure(1, "workspace does not exist");
        });

        let error = backend
            .flatten_workspace_tree(&"missing".to_string())
            .expect_err("nonzero exit should fail");
        let message = format!("{error:#}");
        assert!(
            message.contains("flatten-workspace-tree"),
            "unexpected error: {message}"
        );
        assert!(
            message.contains("workspace does not exist"),
            "unexpected error: {message}"
        );
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn query_command_fails_on_invalid_stdout_json() {
        let (backend, handle) = connect_handshaken(|peer| {
            let _ = peer.read_request();
            peer.write_success("not-json");
        });

        let error = backend
            .list_apps()
            .expect_err("invalid stdout JSON should fail");
        let message = format!("{error:#}");
        assert!(
            message.contains("Failed to execute list-apps"),
            "unexpected error: {message}"
        );
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn read_response_fails_on_invalid_utf8_payload() {
        let (backend, handle) = connect_handshaken(|peer| {
            let _ = peer.read_request();
            peer.write_payload(&[0xff, 0xfe, 0xfd]);
        });

        let error = backend.list_apps().expect_err("invalid UTF-8 should fail");
        let message = format!("{error:#}");
        assert!(
            message.contains("Failed to execute list-apps"),
            "unexpected error: {message}"
        );
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn move_node_to_workspace_sends_window_and_workspace_args() {
        let (tx, rx) = mpsc::channel();
        let (backend, handle) = connect_handshaken(move |peer| {
            tx.send(peer.read_request()).unwrap();
            peer.write_success("");
        });

        backend
            .move_node_to_workspace(&"main".to_string(), 42)
            .expect("move should succeed");
        let request = rx.recv().expect("peer should receive a request");

        assert_eq!(
            request_args(&request),
            ["move-node-to-workspace", "--window-id", "42", "--", "main"]
        );
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn layout_sends_workspace_and_root_layout() {
        let (tx, rx) = mpsc::channel();
        let (backend, handle) = connect_handshaken(move |peer| {
            tx.send(peer.read_request()).unwrap();
            peer.write_success("");
        });

        backend
            .layout(&"coding".to_string(), &AerospaceLayout::VAccordion)
            .expect("layout should succeed");
        let request = rx.recv().expect("peer should receive a request");

        assert_eq!(
            request_args(&request),
            ["layout", "--workspace", "coding", "--root", "v_accordion"]
        );
        handle.join().expect("peer thread should finish");
    }

    #[test]
    fn flatten_workspace_tree_sends_workspace_arg() {
        let (tx, rx) = mpsc::channel();
        let (backend, handle) = connect_handshaken(move |peer| {
            tx.send(peer.read_request()).unwrap();
            peer.write_success("");
        });

        backend
            .flatten_workspace_tree(&"1".to_string())
            .expect("flatten should succeed");
        let request = rx.recv().expect("peer should receive a request");

        assert_eq!(
            request_args(&request),
            ["flatten-workspace-tree", "--workspace", "1"]
        );
        handle.join().expect("peer thread should finish");
    }
}
