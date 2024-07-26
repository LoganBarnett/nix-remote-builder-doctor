use crate::{
  error::AppError,
  test::{MachineTestContext, Test, TestResult},
};

pub struct NixStorePermissionTest {

}

impl Test for NixStorePermissionTest {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError> {
    Ok(TestResult::default(context,  "Nix Store Permission"))
  }
}
