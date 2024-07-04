use crate::{command::command_with_stdin, error::AppError, test::{MachineTestContext, Test, TestResult, TestStatus}};

pub struct MatchingKeysTest {

}

impl Test for MatchingKeysTest  {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    let test_data = "test encryption round trip";
    let encrypt_output = command_with_stdin(
      "rage",
      vec!(
        "--encrypt",
        "--output",
        "-",
        "--recipients-file",
        &(context.machine.private_key_path.clone() + ".pub"),
      ),
      test_data.as_bytes(),
    )?;
    if !encrypt_output.status.success() {
      Ok(TestResult {
        context: context.clone(),
        reason: format!(
          "Encryption failed: {:?}",
          String::from_utf8_lossy(&encrypt_output.stderr),
        ),
        status: TestStatus::Fail,
        suggestion: format!(
          "learn2encrypt",
        ).to_string(),
        test_name: "MatchingKeysTest".to_string(),
      })
    } else {
      let decrypt_output = command_with_stdin(
        "rage",
        vec!(
          "--decrypt",
          "--identity",
          // This is probably wrong, but we probably have an error we can test
          // for or log about here.
          // &ssh_config_value(
          //   "identityfile",
          //   context
          //     .machine
          //     .url
          //     .host_str()
          //     .ok_or(AppError::SshConfigQueryHostnameMissingError(
          //       context.machine.url.clone(),
          //     ))?,
          // )?,
          &context.machine.private_key_path,
        ),
        &encrypt_output.stdout,
      )?;
      if !decrypt_output.status.success() {
        Ok(TestResult {
          context: context.clone(),
          reason: format!(
            "Decryption failed trying to get '{:?}': {:?}",
            test_data,
            String::from_utf8_lossy(&decrypt_output.stderr),
          ).to_string(),
          status: TestStatus::Fail,
          suggestion: format!(
            "Fix it fix it fix it.",
          ).to_string(),
          test_name: "MatchingKeysTest".to_string(),
        })
      } else {
        let decrypted_data = String::from_utf8_lossy(&decrypt_output.stdout);
        if decrypted_data == test_data {
          Ok(TestResult {
            context: context.clone(),
            reason: "".to_string(),
            status: TestStatus::Pass,
            suggestion: "".to_string(),
            test_name: "MatchingKeysTest".to_string(),
          })
        } else {
          Ok(TestResult {
            context: context.clone(),
            reason: format!(
              "Decrypted data {:?} does not match input {:?}.",
              decrypted_data,
              test_data,
            ).to_string(),
            status: TestStatus::Fail,
            suggestion: format!(
              "Come up with something.",
            ).to_string(),
            test_name: "MatchingKeysTest".to_string(),
          })
        }
      }
    }
  }
}
