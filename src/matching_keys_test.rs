/**
 * This test confirms that the public key for the host is also the public key
 * found during a key-scan.  Key scans can return multiple keys, but we just
 * need one of them to match.
 */
use crate::{
  command::command_with_stdin,
  error::AppError,
  ssh_utils::{KeyscanEntry, ssh_keyscan},
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
        let machine_key = KeyscanEntry::parse(
          &context.machine.host_public_key,
        )?;
        if keyscan_entries
          // For now, until I can figure out the borrowing of a nested loop.
          .clone()
          .into_iter()
          .any(|entry| entry.key_data == machine_key.key_data) {
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
                machine_key.key_data,
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
        match e {
          AppError::SshKeyscanCommandSigPipeError(_) => {
            Ok(TestResult::Fail(FailData {
            context: context.clone(),
            test_name: "MatchingKeysTest".into(),
            reason: format!("{:?}", e),
            suggestion:
              // TODO: Have this automatically wrapped to 80 chars.
              "ssh-keyscan failed with SIGPIPE and that means the remote sshd \
               instance disconnected with a preauth failure (see sshd logs for \
               details).  This means the host key type is incorrect.  This \
               test assumes ed25519 as the type.".into(),
          }))
          },
          _ => Ok(TestResult::Fail(FailData {
            context: context.clone(),
            test_name: "MatchingKeysTest".into(),
            reason: format!("{:?}", e),
            suggestion: "Unknown error running ssh-keyscan.".into(),
          })),
        }
      })
  }
}
