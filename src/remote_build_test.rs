use crate::{error::AppError, ssh::Ssh, ssh2_adapter::Ssh2, test::{MachineTestContext, Test, TestResult}};

pub struct RemoteBuildTest {

}

impl Test for RemoteBuildTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    let mut ssh = Ssh2::new();
    ssh.connect(&context.machine)
      .and_then(|()| {
        println!("Connected in remote build test.  Running...");
        let output = ssh.run("nix build nixpkgs#hello")?;
        ssh.disconnect()?;
        if output.status == 0 {
          Ok(TestResult {
            pass: true,
            reason: "".to_string(),
            context: context.clone(),
          })
        } else {
          Ok(TestResult {
            pass: false,
            // TODO: Should be stderr or both.
            reason: output.stdout,
            context: context.clone(),
          })
        }
      })
      .or_else(|e| {
        println!("Connection failure!");
        Ok(TestResult {
          pass: false,
          reason: format!("{:?}", e),
          context: context.clone(),
        })
      })
  }
}
