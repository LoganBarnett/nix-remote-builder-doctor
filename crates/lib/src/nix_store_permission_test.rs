use crate::{
  error::AppError,
  test::{MachineTestContext, Test, TestResult},
};

pub struct NixStorePermissionTest {}

impl Test for NixStorePermissionTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    Ok(TestResult::not_implemented(context, "Nix Store Permission"))
  }
}
