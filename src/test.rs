use crate::{error::AppError, Machine};

#[derive(Clone)]
pub struct AppTestContext {

}

#[derive(Clone)]
pub struct TestResult {
  pub pass: bool,
  pub reason: String,
  pub context: MachineTestContext,
}

#[derive(Clone)]
pub struct MachineTestContext {
  pub machine: Machine,
  pub app_context: AppTestContext,
}

#[derive(Clone)]
pub struct MachineTestResults {
  pub machine: Machine,
  pub test_results: Vec<TestResult>,
}

pub struct AppTestResults {
  pub machine_test_results: Vec<MachineTestResults>,
}

pub trait Test {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError>;
}
