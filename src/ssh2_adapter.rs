// The adapter for the ssh2 library.  I started with this one but it seems to be
// abandoned or in a maintenance mode, and I'm having problems that already-open
// tickets have.
use log::*;
use std::io::prelude::*;
use std::{net::TcpStream, path::Path};
use ssh2::{Session, TraceFlags};

use crate::{machine::Machine, ssh::{CommandOutput, Ssh}, AppError};

pub struct Ssh2 {
  pub authenticated: bool,
  pub session: Session,
}

impl Ssh2 {

  pub fn new() -> Ssh2 {
    Ssh2 {
      authenticated: false,
      session: Session::new().unwrap(),
    }
  }

}

impl Ssh for Ssh2 {

  fn connect(&mut self, machine: &Machine) -> Result<(), AppError> {
    let host = machine.url.host_str().ok_or(AppError::HostMissingError())?;
    let tcp = TcpStream::connect(
      format!("{}:{}", host, machine.url.port().unwrap_or(22)),
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
      Some(&machine.public_key),
      &machine.private_key,
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
    // TODO: Clean up unwraps.
    let mut channel = self.session.channel_session()
      .map_err(AppError::SshChannelOpenFailure)?;
    channel.exec(command).unwrap();
    let mut s = String::new();
    // Is this just stdout?
    channel.read_to_string(&mut s).unwrap();
    trace!("Command result: {}", s);
    channel.wait_close().unwrap();
    let exit_status = channel.exit_status().unwrap();
    debug!("Exit status: {}", exit_status);
    Ok(CommandOutput {
      // This is actually a u8 but I'd have to change the interface.  Do that
      // later.
      status: exit_status as u16,
      stdout: s,
      stderr: "".to_string(),
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
}
