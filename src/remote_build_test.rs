use log::*;

use crate::{
  error::AppError,
  ssh::Ssh,
  ssh2_adapter::Ssh2,
  test::{FailData, MachineTestContext, PassData, Test, TestResult},
};

pub struct RemoteBuildTest {

}

impl Test for RemoteBuildTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    let mut ssh = Ssh2::new();
    ssh.connect(&context.machine)
      .and_then(|()| {
        info!("Connected in remote build test.  Running...");
        let output = ssh.run("nix build nixpkgs#hello")?;
        ssh.disconnect()?;
        match output.status {
          0 => Ok(TestResult::Pass(PassData {
            context: context.clone(),
            test_name: "Remote Build".to_string(),
          })),
          _ => Ok(TestResult::Fail(FailData {
            // TODO: Should be stderr or both.
            reason: format!(
              "exit code: {}\nstdout:{}\nstderr: {}",
              output.status,
              output.stdout,
              output.stderr,
            ),
            context: context.clone(),
            suggestion: "No suggestions yet.".to_string(),
            test_name: "Remote Build".to_string(),
          })),
        }
      })
      .or_else(|e| {
        error!("Connection failure! {:?}", e);
        Ok(TestResult::Fail(FailData {
          reason: format!("{:?}", e),
          context: context.clone(),
          suggestion: "No suggestions yet.".to_string(),
          test_name: "Remote Build".to_string(),
        }))
      })
  }
}
