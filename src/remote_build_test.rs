use crate::{error::AppError, ssh::Ssh, ssh2_adapter::Ssh2, test::{MachineTestContext, Test, TestResult, TestStatus}};

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
            reason: "".to_string(),
            status: TestStatus::Pass,
            context: context.clone(),
          })
        } else {
          Ok(TestResult {
            // TODO: Should be stderr or both.
            reason: output.stdout,
            status: TestStatus::Fail,
            context: context.clone(),
          })
        }
      })
      .or_else(|e| {
        println!("Connection failure!");
        Ok(TestResult {
          reason: format!("{:?}", e),
          status: TestStatus::Fail,
          context: context.clone(),
        })
      })
  }
}
