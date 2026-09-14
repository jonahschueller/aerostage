use std::env;
use std::io;
use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AerospaceBackendError {
    #[error("$USER environment variable is not set")]
    UnknownUser(#[from] env::VarError),

    #[error("unable to connect to AeroSpace at {path}")]
    UnavailableAerospace {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("I/O error on AeroSpace socket: {0}")]
    Io(#[from] io::Error),

    #[error(
        "AeroSpace protocol version mismatch (client: {client_version}, server: {server_version})"
    )]
    IncompatibleVersion {
        client_version: u32,
        server_version: u32,
    },

    #[error("AeroSpace response is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("failed to serialize AeroSpace request: {0}")]
    Serialize(#[source] serde_json::Error),

    #[error("failed to deserialize AeroSpace response: {0}")]
    Deserialize(#[source] serde_json::Error),

    #[error("AeroSpace command `{command}` failed (exit {exit_code}): {stderr}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },
}

pub type Result<T> = std::result::Result<T, AerospaceBackendError>;
