use crate::{error::AppError, test::{MachineTestContext, Test, TestResult}};

pub struct LocalToRemoteBuildTest {

}

impl Test for LocalToRemoteBuildTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    Ok(TestResult::default(context, "Local To Remote Build"))
  }
}
