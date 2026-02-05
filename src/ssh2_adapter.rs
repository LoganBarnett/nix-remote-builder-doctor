// The adapter for the ssh2 library.  I started with this one but it seems to be
// abandoned or in a maintenance mode, and I'm having problems that already-open
// tickets have.
use log::*;
use std::io::prelude::*;
use std::net::TcpStream;
use ssh2::Session;

use crate::{machine::Machine, ssh::{CommandOutput, Ssh}, AppError};

pub struct Ssh2 {
  pub authenticated: bool,
  pub session: Session,
  pub hostname: Option<String>,
  pub port: Option<u16>,
}

impl Ssh2 {

  pub fn new() -> Ssh2 {
    Ssh2 {
      authenticated: false,
      session: Session::new().unwrap(),
      hostname: None,
      port: None,
    }
  }

}

impl Ssh for Ssh2 {

  fn connect(&mut self, machine: &Machine) -> Result<(), AppError> {
    let host = machine.url.host_str().ok_or(AppError::HostMissingError())?;
    let port = machine.url.port().unwrap_or(22);

    // Store hostname and port for later use (e.g., host key checking)
    self.hostname = Some(host.to_string());
    self.port = Some(port);

    let tcp = TcpStream::connect(
      format!("{}:{}", host, port),
    )
      .map_err(AppError::HostConnectionFailedError)?;
    // let mut sesh = Session::new()
    //   .map_err(AppError::SshSessionCreationError)
    //   ?;
    self.session.set_tcp_stream(tcp);
    self.session.set_timeout(5000);
    // sesh.trace(TraceFlags::all());
    self.session.set_blocking(true);
    debug!("Starting handshake for {}...", host);
    self.session
      .handshake()
      .map_err(AppError::SshSessionHandshakeError)
      ?;
    debug!("Handshake for {} done!", host);
    debug!("Starting authentication for {}...", host);
    // sesh.userauth_pubkey_file(
    //   machine.url.username(),
    //   Some(Path::new(format!("{}.pub", &machine.public_key).as_str())),
    //   Path::new(&machine.private_key_path),
    //   None,
    // )
    //     .map_err(AppError::SshSessionAuthError)?;
    // This seems to be bugged.  Possibly
    // https://github.com/alexcrichton/ssh2-rs/issues/284 but the error isn't the
    // same. I get:
    // { code: Session(-5), msg: \"Unable to exchange encryption keys\" }
    // It's the same as doing it with the file, so something else must be amiss.
    // This has the same error, but it's conditions don't look promising:
    // https://github.com/alexcrichton/ssh2-rs/issues/254
    self.session.userauth_pubkey_memory(
      machine.url.username(),
      // TODO: This seems to work, but we could also get the public key.  This
      // should be the local public key, not the host public key.
      None,
      &machine.user_private_key,
      None,
    )
      .map_err(AppError::SshSessionAuthError)?;
    debug!("Authentication for {} done!", host);
    debug!("Authenticated for {}? {}", host, self.session.authenticated());
    self.authenticated = self.session.authenticated();
    Ok(())
  }

  fn run(&self, command: &str) -> Result<CommandOutput, AppError> {
    debug!("Starting command: {}", command);
    let mut channel = self.session.channel_session()
      .map_err(AppError::SshChannelOpenFailure)?;
    let _ = channel
      .exec(command)
      .map_err(AppError::SshCommandExecuteError)?;
    let mut stdout = String::new();
    // This is stdout by default.  There's a stderr on the Channel type as well
    // (see below for its use).
    let _ = channel
      .read_to_string(&mut stdout)
      .map_err(AppError::SshChannelReadError)?;
    let mut stderr = String::new();
    let _ = channel
      .stderr()
      .read_to_string(&mut stderr)
      .map_err(AppError::SshChannelReadError)?;
    trace!("Command stdout: {}", stdout);
    trace!("Command stderr: {}", stderr);
    channel.wait_close().map_err(AppError::SshChannelCloseError)?;
    let exit_status = channel
      .exit_status()
      .map_err(AppError::SshCommandNotTerminatedError)?;
    debug!("Exit status: {}", exit_status);
    Ok(CommandOutput {
      // This is actually a u8 but I'd have to change the interface.  Do that
      // later.
      status: exit_status as u16,
      stdout,
      stderr,
    })
  }

  fn disconnect(self) -> Result<(), AppError> {
    self.session.disconnect(
      // Is this correct?
      Some(ssh2::DisconnectCode::ByApplication),
      "Done!",
      None,
    )
    .map_err(AppError::SshSessionDisconnectError)
  }

  fn is_authenticated(&self) -> bool {
    self.authenticated
  }

  fn check_host_key(&self) -> Result<bool, AppError> {
    use ssh2::{CheckResult, KnownHostFileKind};
    use std::path::PathBuf;

    // Get the hostname and port from the stored values
    let hostname = self.hostname.as_ref()
      .ok_or_else(|| AppError::SshHostKeyError(
        ssh2::Error::from_errno(ssh2::ErrorCode::Session(-1))
      ))?;
    let port = self.port.unwrap_or(22);

    // Get the host key from the session
    let (host_key, _key_type) = self.session.host_key()
      .ok_or_else(|| AppError::SshHostKeyError(
        ssh2::Error::from_errno(ssh2::ErrorCode::Session(-1))
      ))?;

    // Create known_hosts checker
    let mut known_hosts = self.session.known_hosts()
      .map_err(AppError::SshKnownHostsError)?;

    // Try to read the user's known_hosts file
    let mut known_hosts_path = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    known_hosts_path.push(".ssh");
    known_hosts_path.push("known_hosts");

    // Read the known_hosts file if it exists
    if known_hosts_path.exists() {
      debug!("Reading known_hosts from: {:?}", known_hosts_path);
      known_hosts.read_file(&known_hosts_path, KnownHostFileKind::OpenSSH)
        .map_err(AppError::SshKnownHostsFileReadError)?;
    } else {
      debug!("No known_hosts file found at: {:?}", known_hosts_path);
      // No known_hosts file, so the check will fail
      return Ok(false);
    }

    // Format the hostname with port if non-standard
    let host_to_check = if port != 22 {
      format!("[{}]:{}", hostname, port)
    } else {
      hostname.clone()
    };

    // Check the host key against known_hosts
    let check_result = known_hosts.check(&host_to_check, host_key);

    debug!("Host key check result for {}: {:?}", host_to_check, check_result);

    match check_result {
      CheckResult::Match => Ok(true),
      CheckResult::NotFound => {
        debug!("Host key not found in known_hosts for: {}", host_to_check);
        Ok(false)
      },
      CheckResult::Mismatch => {
        warn!("Host key mismatch detected for: {}", host_to_check);
        Ok(false)
      },
      CheckResult::Failure => {
        debug!("Host key check failed for: {}", host_to_check);
        Ok(false)
      },
    }
  }

}
