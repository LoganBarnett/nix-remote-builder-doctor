use crate::{error::AppError, test::{MachineTestContext, Test, TestResult, TestStatus}};

pub struct NixStorePermissionTest {

}

impl Test for NixStorePermissionTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    Ok(TestResult {
      context: context.clone(),
      reason: "".to_string(),
      status: TestStatus::Fail,
      suggestion: "".to_string(),
      test_name: "Nix Store Permission".to_string(),
    })
  }
}
