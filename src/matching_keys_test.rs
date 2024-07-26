/**
 * This test confirms that the public key for the host is also the public key
 * found during a key-scan.  Key scans can return multiple keys, but we just
 * need one of them to match.
 */
use crate::{
  command::command_with_stdin,
  error::AppError,
  ssh_utils::ssh_keyscan,
  test::{
    MachineTestContext,
    Test,
    TestResult,
    PassData,
    FailData,
  }
};

pub struct MatchingKeysTest {

}

impl Test for MatchingKeysTest  {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    ssh_keyscan(
      context.machine.url.host_str().unwrap(),
      context.machine.url.port().unwrap(),
    )
      .and_then(|keyscan_entries| {
        if keyscan_entries
          // For now, until I can figure out the borrowing of a nested loop.
          .clone()
          .into_iter()
          .any(|entry| {
            entry.key_data == context.machine.host_public_key
          }) {
            Ok(TestResult::Pass(PassData {
              context: context.clone(),
              test_name: "MatchingKeysTest".into(),
            }))
          } else {
            Ok(TestResult::Fail(FailData {
              context: context.clone(),
              test_name: "MatchingKeysTest".into(),
              reason: format!(
                "None of the keys from the host matched the key found in \
                 /etc/machines/nix.  \n Machine key:\n{}\n Scanned keys:\n{}",
                context.machine.host_public_key,
                keyscan_entries
                  .into_iter()
                  .map(|ks| {
                    format!("  {}", ks.key_data)
                  })
                  .collect::<Vec<_>>()
                  .join("\n"),
              ),
              suggestion: "Figure it out".into(),
            }))
          }
      })
      .or_else(|e| {
        Ok(TestResult::Fail(FailData {
          context: context.clone(),
          test_name: "MatchingKeysTest".into(),
          reason: format!("{:?}", e),
          suggestion: "Figure it out".into(),
        }))
      })
  }
}
