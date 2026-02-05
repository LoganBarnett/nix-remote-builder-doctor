use crate::{
  error::AppError,
  ssh::Ssh,
  ssh2_adapter::Ssh2,
  test::{FailData, MachineTestContext, PassData, Test, TestResult},
};
use log::*;

pub struct HostKeyTest {

}

impl Test for HostKeyTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    debug!("Checking host key for {}...", context.machine.url.to_string());
    let mut ssh = Ssh2::new();
    ssh.connect(&context.machine)
      .and_then(|()| {
        let host_key_valid = ssh.check_host_key()?;
        ssh.disconnect()?;
        trace!(
          "Host key check for {}: {}",
          context.machine.url,
          host_key_valid,
        );
        Ok(if host_key_valid {
          TestResult::Pass(PassData {
            context: context.clone(),
            test_name: "HostKey".to_string(),
          })
        } else {
          TestResult::Fail(FailData {
            context: context.clone(),
            reason: "Host key not found in known_hosts or mismatch detected".to_string(),
            suggestion: format!(
              "Add the host key to your known_hosts file. You can do this by running:\n{}",
              context.machine.ssh_invocation(),
            ).to_string(),
            test_name: "HostKey".to_string(),
          })
        })
      })
      .or_else(|e| {
        Ok(TestResult::Fail(FailData {
          context: context.clone(),
          reason: format!("{:?}", e),
          suggestion: format!(
            "Ensure the host key is in your known_hosts file. \
             Run the following command to add it:\n{}",
            context.machine.ssh_invocation(),
          ).to_string(),
          test_name: "HostKey".to_string(),
        }))
      })
  }
}
