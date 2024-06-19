use std::str::Utf8Error;


#[derive(Debug)]
pub enum AppError {
  HostConnectionFailedError(std::io::Error),
  HostMissingError(),
  MachinesFileReadError(std::io::Error),
  MachinesEntryMissingFieldError(String),
  MachinesEntryNotNumberFieldError(String),
  PrivateKeyFileReadError(std::io::Error),
  PublicKeyDecodeError(base64::DecodeError),
  PublicKeyUtf8Error(Utf8Error),
  SshChannelOpenFailure(ssh2::Error),
  SshConfigQueryError(String),
  SshSessionDisconnectError(ssh2::Error),
  SshSessionAuthError(ssh2::Error),
  // SshSessionCreationError(ssh2::Error),
  SshSessionHandshakeError(ssh2::Error),
  SshSpawnProcessError(std::io::Error),
  UrlParseError(url::ParseError),
  UrlHostnameMissingError(String),
}
