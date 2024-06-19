// I've found SSH options online to be... lacking.  So this is the home rolled
// version.

pub struct SshHome {

}

impl Ssh for Russh {

  fn connect(&self) -> Result<(), AppError> {
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
