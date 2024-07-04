use std::str::Utf8Error;

use log::SetLoggerError;
use url::Url;


#[derive(Debug)]
pub enum AppError {
  HostConnectionFailedError(std::io::Error),
  HostMissingError(),
  LoggingInitializationError(SetLoggerError),
  MachinesEntryMissingFieldError(String),
  MachinesEntryNotNumberFieldError(String),
  MachinesEntryUrlHostnameMissingError(String),
  MachinesFileReadError(std::io::Error),
  PrivateKeyFileReadError(std::io::Error),
  PublicKeyFileReadError(std::io::Error),
  PublicKeyDecodeError(base64::DecodeError),
  PublicKeyUtf8Error(Utf8Error),
  RageSpawnProcessError(std::io::Error),
  RageSpawnProcessStdinError(),
  SshChannelOpenFailure(ssh2::Error),
  SshConfigQueryError(String),
  SshConfigQueryFieldMissingError(String, String),
  SshConfigQueryHostnameMissingError(Url),
  SshSessionDisconnectError(ssh2::Error),
  SshSessionAuthError(ssh2::Error),
  // SshSessionCreationError(ssh2::Error),
  SshSessionHandshakeError(ssh2::Error),
  SshSpawnProcessError(std::io::Error),
  UrlParseError(url::ParseError),
  UrlHostnameMissingError(String),
}
