use crate::{error::AppError, Machine};
use strum::Display;
use tap::Tap;
use log::*;

#[derive(Clone)]
pub struct AppTestContext {

}

#[derive(Clone)]
pub struct TestResult {
  pub context: MachineTestContext,
  pub reason: String,
  pub status: TestStatus,
  pub suggestion: String,
  pub test_name: String,
}

#[derive(Clone)]
pub struct MachineTestContext {
  pub machine: Machine,
  pub app_context: AppTestContext,
}

#[derive(Clone)]
pub struct MachineTestResult {
  pub machine: Machine,
  pub test_results: Vec<TestResult>,
}

pub struct AppTestResults {
  pub machine_test_results: Vec<MachineTestResult>,
}

pub trait Test {
  fn test(&self, context: &MachineTestContext) -> Result<TestResult, AppError>;
}

#[derive(Clone, Debug, Display, PartialEq)]
pub enum TestStatus {
  Pass,
  Fail,
}

impl TestStatus {

  pub fn from(b: bool) -> TestStatus {
    if b {
      TestStatus::Pass
    } else {
      TestStatus::Fail
    }
      .tap(|x| trace!("{:?} to {:?}", b, x))
  }

}
