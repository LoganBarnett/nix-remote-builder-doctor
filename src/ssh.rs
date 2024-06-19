use crate::{error::AppError, Machine};

pub trait Ssh {

  fn connect(
    &mut self,
    machine: &Machine,
  ) -> Result<(), AppError>;

  fn run(&self, command: &str) -> Result<CommandOutput, AppError>;

  fn disconnect(self) -> Result<(), AppError>;

  fn is_authenticated(&self) -> bool;

}

pub struct CommandOutput {
  pub status: u16,
  pub stdout: String,
  pub stderr: String,
}

impl CommandOutput {

}
