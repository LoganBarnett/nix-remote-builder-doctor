use crate::{
  dns_utils::resolved_host,
  error::AppError,
  test::{FailData, MachineTestContext, PassData, Test, TestResult},
};

use log::*;

pub struct DnsTest {

}

impl Test for DnsTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    debug!("Resolving host \"{}\"...", context.machine.url.to_string());
    Ok(
      match resolved_host(&context.machine.url.to_string()) {
        Ok(_) => TestResult::Pass(PassData {
          context: context.clone(),
          test_name: "DNS".into(),
        }),
        Err(_) => TestResult::Fail(FailData {
          context: context.clone(),
          reason: "Reason not supported for this error.".into(),
          suggestion: "See if another host on the network can resolve this host, \
                       or check the host directly.".into(),
          test_name: "DNS".into(),
        }),
      }
    )
  }
}
