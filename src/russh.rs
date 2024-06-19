// russh uses tokio, and I just don't want to pull that in here.  Not without
// some really powerful convincing.
// use russh::Session;

use crate::{machine::Machine, ssh::Ssh, AppError};

pub struct Russh {

}

impl Ssh for Russh {

  fn connect(&self, machine: &Machine) -> Result<(), AppError> {
    // Session::connect(
    //   cli.private_key,
    //   cli.username.unwrap_or("root".to_string()),
    //   (cli.host, cli.port),
    // )
    Ok(())
  }

  fn run(&self, command: String) -> Result<(), AppError> {
    Ok(())
  }

  fn disconnect(self) -> Result<(), AppError> {
    Ok(())
  }

}
