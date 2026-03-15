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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_helpers::*;

    #[test]
    #[ignore]  // Requires network access - run with: cargo test -- --ignored
    fn test_dns_resolution_valid_host() {
        // Use a well-known host that should resolve even in sandboxed environments
        let context = create_test_context("8.8.8.8");
        let dns_test = DnsTest {};

        let result = dns_test.test(&context).unwrap();
        assert_test_passes(&result, "DNS");
    }

    #[test]
    #[ignore]  // Requires network access - run with: cargo test -- --ignored
    fn test_dns_resolution_invalid_host() {
        let context = create_test_context("this-host-definitely-does-not-exist.invalid");
        let dns_test = DnsTest {};

        let result = dns_test.test(&context).unwrap();
        assert_test_fails(&result, "DNS");
    }
}
