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
    match context.machine.url.host_str() {
      Some(host_str) => {
        debug!("Resolving host \"{}\"...", host_str);
        Ok(
          match resolved_host(&host_str) {
            Ok(_) => TestResult::Pass(PassData {
              context: context.clone(),
              test_name: "DNS".into(),
            }),
            Err(_) => TestResult::Fail(FailData {
              context: context.clone(),
              reason: "Reason not yet supported for this error.".into(),
              suggestion: "See if another host on the network can resolve this \
                           host, or check the host directly.".into(),
              test_name: "DNS".into(),
            }),
          }
        )
      },
      None => {
        Ok(TestResult::Fail(FailData {
          context: context.clone(),
          reason: format!(
            "Could not get host name from URL {}",
            context.machine.url.to_string(),
          ),
          suggestion: "Check the URL to make sure it is correct.".into(),
          test_name: "DNS".into(),
        }))
      }
    }
  }
}
