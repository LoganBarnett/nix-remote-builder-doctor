use crate::{
  error::AppError,
  ssh::Ssh,
  ssh2_adapter::Ssh2,
  test::{FailData, MachineTestContext, PassData, Test, TestResult},
};
use log::*;

pub struct ConnectionTest {

}

impl Test for ConnectionTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    debug!("Connecting to {}...", context.machine.url.to_string());
    let mut ssh = Ssh2::new();
    ssh.connect(&context.machine)
      .and_then(|()| {
        let authenticated = ssh.is_authenticated();
        ssh.disconnect()?;
        trace!(
          "In test: authenticated for {}: {}",
          context.machine.url,
          authenticated,
        );
        Ok(if authenticated {
          TestResult::Pass(PassData {
            context: context.clone(),
            test_name: "Connection".to_string(),
          })
        } else {
          TestResult::Fail(FailData {
            context: context.clone(),
            reason: format!("authenticated: {}", authenticated).to_string(),
            suggestion: "".to_string(),
            test_name: "Connection".to_string(),
          })
        })
      })
      .or_else(|e| {
        Ok(TestResult::Fail(FailData {
          context: context.clone(),
          reason: format!("{:?}", e),
          suggestion: format!(
            "Use the following to test your ssh connection, \
             and add -v's to increase verobsity (ie, -vvv):\n{}",
            context.machine.ssh_invocation(),
          ).to_string(),
          test_name: "Connection".to_string(),
        }))
      })
  }
}
