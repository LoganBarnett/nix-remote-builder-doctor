use crate::{
  error::AppError,
  test::{
    FailData,
    MachineTestContext,
    PassData,
    Test,
    TestResult
  },
};
use std::process::Command;

pub struct LocalToRemoteBuildTest {

}

impl Test for LocalToRemoteBuildTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    let command_result = Command::new("nix")
      .arg("build")
      .arg("--builders")
      .arg(context.machine.etc_machines_entry())
      .arg("nixpkgs#hello")
      .output()
      ;
    match command_result {
      Ok(output) => {
        Ok(match output.status.code() {
          Some(code) => match code {
            0 => TestResult::Pass(PassData {
              context: context.clone(),
              test_name: "LocalToRemoteBuildTest".into(),
            }),
            _ => TestResult::Fail(FailData {
              context: context.clone(),
              reason: String::from_utf8_lossy(&output.stderr).to_string(),
              suggestion: "No suggestions yet.".into(),
              test_name: "LocalToRemoteBuildTest".into(),
            }),
          },
          None => TestResult::Fail(FailData {
            context: context.clone(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
            suggestion: "No suggestions yet.".into(),
            test_name: "LocalToRemoteBuildTest".into(),
          }),
        })
      },
      Err(e) => {
        Ok(TestResult::Fail(FailData {
          context: context.clone(),
          reason: e.to_string(),
          suggestion: "No suggestions yet.".into(),
          test_name: "LocalToRemoteBuildTest".into(),
        }))
      },
    }
  }
}
