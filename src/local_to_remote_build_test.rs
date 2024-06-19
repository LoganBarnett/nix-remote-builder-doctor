use crate::{error::AppError, test::{Test, MachineTestContext, TestResult}};

pub struct LocalToRemoteBuildTest {

}

impl Test for LocalToRemoteBuildTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    Ok(TestResult {
      pass: false,
      reason: "".to_string(),
      context: context.clone(),
    })
  }
}
