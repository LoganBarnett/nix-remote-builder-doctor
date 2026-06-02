use std::str::Utf8Error;
use thiserror::Error;

/// Semantic errors raised across the doctor's tests, transports, and
/// supporting helpers.  Each variant names the *operation* that failed
/// rather than the underlying I/O class so call sites stay
/// self-documenting; see `llms.org` § "Error Handling" for the rule.
#[derive(Debug, Error)]
pub enum AppError {
  #[error("Age decryption failed: {0}")]
  AgeDecryptionFailure(String),
  #[error("Age encryption failed: {0}")]
  AgeEncryptionFailure(String),
  #[error("Recipients path is not valid UTF-8")]
  AgeRecipientsPathInvalidUtf8Error,
  #[error("Identity (private key) path is not valid UTF-8")]
  AgeIdentityPathInvalidUtf8Error,
  #[error("nslookup exited with status {0}: {1}")]
  DnsCommandFailedError(i32, String),
  #[error("nslookup terminated by signal {0}: {1}")]
  DnsCommandSignalError(String, String),
  #[error("nslookup terminated by SIGPIPE: {0}")]
  DnsCommandSigPipeError(String),
  #[error("Failed to spawn nslookup: {0}")]
  DnsCommandSpawnProcessError(std::io::Error),
  #[error("nslookup terminated by unrecognised signal number {0}")]
  DnsCommandUnknownSignalError(i32),
  #[error("nslookup terminated by neither exit code nor signal")]
  DnsCommandNoSignalError,
  #[error("Failed to open TCP connection to host: {0}")]
  HostConnectionFailedError(std::io::Error),
  #[error("Machine URL has no host component")]
  HostMissingError,
  #[error("Machine has no host public key")]
  HostPublicKeyMissingError,
  #[error("Machines file entry is missing a required field: {0}")]
  MachinesEntryMissingFieldError(String),
  #[error("Machines file entry has a non-numeric field: {0}")]
  MachinesEntryNotNumberFieldError(String),
  #[error("Machines file entry URL is missing a hostname: {0}")]
  MachinesEntryUrlHostnameMissingError(String),
  #[error("Failed to read machines file: {0}")]
  MachinesFileReadError(std::io::Error),
  #[error("Machine URL is missing the port that the SSH transport needs")]
  MachinePortMissingError,
  #[error("Test is not implemented")]
  NotImplementedError,
  #[error("Failed to serialize doctor results as JSON: {0}")]
  JsonSerializationError(serde_json::Error),
  #[error("Failed to read user's private key file {1}: {0}")]
  PrivateKeyFileReadError(std::io::Error, String),
  #[error("Failed to read host's public key file {1}: {0}")]
  PublicKeyFileReadError(std::io::Error, String),
  #[error("Failed to base64-decode public key: {0}")]
  PublicKeyDecodeError(base64::DecodeError),
  #[error("Failed to UTF-8 decode public key bytes: {0}")]
  PublicKeyUtf8Error(Utf8Error),
  #[error("Failed to spawn rage: {0}")]
  RageSpawnProcessError(std::io::Error),
  #[error("Failed to acquire rage's stdin pipe")]
  RageSpawnProcessStdinError,
  #[error("Failed to write to rage's stdin: {0}")]
  RageStdinWriteError(std::io::Error),
  #[error("Failed to close SSH channel: {0}")]
  SshChannelCloseError(ssh2::Error),
  #[error("Failed to open SSH channel: {0}")]
  SshChannelOpenFailure(ssh2::Error),
  #[error("Failed to read from SSH channel: {0}")]
  SshChannelReadError(std::io::Error),
  #[error("Failed to execute SSH command: {0}")]
  SshCommandExecuteError(ssh2::Error),
  #[error("SSH command did not terminate: {0}")]
  SshCommandNotTerminatedError(ssh2::Error),
  #[error("Failed to query ssh config for host {0}")]
  SshConfigQueryError(String),
  #[error("ssh -G for host {0} did not report field {1}")]
  SshConfigQueryFieldMissingError(String, String),
  #[error("ssh-keyscan exited with status {0}: {1}")]
  SshKeyscanCommandFailedError(i32, String),
  #[error("ssh-keyscan terminated by signal {0}: {1}")]
  SshKeyscanCommandSignalError(String, String),
  #[error("ssh-keyscan terminated by SIGPIPE: {0}")]
  SshKeyscanCommandSigPipeError(String),
  #[error("Failed to spawn ssh-keyscan: {0}")]
  SshKeyscanCommandSpawnProcessError(std::io::Error),
  #[error("ssh-keyscan terminated by unrecognised signal number {0}")]
  SshKeyscanCommandUnknownSignalError(i32),
  #[error("ssh-keyscan terminated by neither exit code nor signal")]
  SshKeyscanCommandNoSignalError,
  #[error("Failed to parse ssh-keyscan output line: {0}")]
  SshKeyscanParseError(String),
  #[error("Failed to disconnect SSH session: {0}")]
  SshSessionDisconnectError(ssh2::Error),
  #[error("Failed to authenticate SSH session: {0}")]
  SshSessionAuthError(ssh2::Error),
  #[error("Failed to construct ssh2 Session: {0}")]
  SshSessionCreateError(ssh2::Error),
  #[error("Failed to perform SSH handshake: {0}")]
  SshSessionHandshakeError(ssh2::Error),
  #[error("Failed to read SSH host key: {0}")]
  SshHostKeyError(ssh2::Error),
  #[error("Failed to query SSH known_hosts: {0}")]
  SshKnownHostsError(ssh2::Error),
  #[error("Failed to read SSH known_hosts file: {0}")]
  SshKnownHostsFileReadError(ssh2::Error),
  #[error("Failed to spawn ssh: {0}")]
  SshSpawnProcessError(std::io::Error),
  #[error("Failed to parse URL: {0}")]
  UrlParseError(url::ParseError),
  #[error("URL is missing a hostname: {0}")]
  UrlHostnameMissingError(String),
}
