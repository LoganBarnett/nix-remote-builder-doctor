use crate::{error::AppError, test::{MachineTestContext, Test, TestResult, TestStatus}};

pub struct LocalToRemoteBuildTest {

}

impl Test for LocalToRemoteBuildTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    Ok(TestResult {
      context: context.clone(),
      reason: "".to_string(),
      status: TestStatus::Fail,
    })
  }
}
