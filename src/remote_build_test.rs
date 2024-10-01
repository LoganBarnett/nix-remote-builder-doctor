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
        let build_command = "nix build nixpkgs#hello";
        let build_output = ssh.run(build_command)?;
        ssh.disconnect()?;
        match build_output.status {
          0 => Ok(TestResult::Pass(PassData {
            context: context.clone(),
            test_name: "Remote Build".into(),
          })),
          _ => Ok(TestResult::Fail(FailData {
            // TODO: Should be stderr or both.
            reason: format!(
              "exit code: {}\nstdout:{}\nstderr: {}",
              build_output.status,
              build_output.stdout,
              build_output.stderr,
            ),
            context: context.clone(),
            suggestion: format!(
              "Use the following to connect to the host:\n{}
Once connected, use the following to trigger a build:\n{}
",
              context.machine.ssh_invocation(),
              build_command,
            ),
            test_name: "Remote Build".into(),
          })),
        }
      })
      .or_else(|e| {
        error!("Connection failure! {:?}", e);
        Ok(TestResult::Fail(FailData {
          reason: format!("{:?}", e),
          context: context.clone(),
          suggestion: "No suggestions yet.".into(),
          test_name: "Remote Build".into(),
        }))
      })
  }
}
