use crate::{error::AppError, ssh::Ssh, ssh2_adapter::Ssh2, test::{MachineTestContext, Test, TestResult, TestStatus}};

pub struct ConnectionTest {

}

impl Test for ConnectionTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    println!("Connecting to {}...", context.machine.url.to_string());
    let mut ssh = Ssh2::new();
    ssh.connect(&context.machine)
      .and_then(|()| {
        let authenticated = ssh.is_authenticated();
        ssh.disconnect()?;
        println!("In test: authenticated for {}: {}", context.machine.url, authenticated);
        Ok(TestResult {
          context: context.clone(),
          // TODO: Maybe just make two variants.
          reason: format!("authenticated: {}", authenticated).to_string(),
          status: TestStatus::from(authenticated),
        })
      })
      .or_else(|e| {
        Ok(TestResult {
          context: context.clone(),
          reason: format!("{:?}", e),
          status: TestStatus::Fail,
        })
      })
  }
}
