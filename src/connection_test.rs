use crate::{error::AppError, ssh::Ssh, ssh2_adapter::Ssh2, test::{MachineTestContext, Test, TestResult, TestStatus}};
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
        Ok(TestResult {
          context: context.clone(),
          // TODO: Maybe just make two variants.
          reason: format!("authenticated: {}", authenticated).to_string(),
          status: TestStatus::from(authenticated),
          suggestion: "".to_string(),
          test_name: "Connection".to_string(),
        })
      })
      .or_else(|e| {
        Ok(TestResult {
          context: context.clone(),
          reason: format!("{:?}", e),
          status: TestStatus::Fail,
          suggestion: format!(
            "Use the following to test your ssh connection, \
             and add -v's to increase verobsity (ie, -vvv):\n{}",
            context.machine.ssh_invocation(),
          ).to_string(),
          test_name: "Connection".to_string(),
        })
      })
  }
}
