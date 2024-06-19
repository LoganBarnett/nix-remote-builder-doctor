use crate::{error::AppError, test::{Test, MachineTestContext, TestResult}};

pub struct NixStorePermissionTest {

}

impl Test for NixStorePermissionTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    Ok(TestResult {
      pass: false,
      reason: "".to_string(),
      context: context.clone(),
    })
  }
}
