use std::any::type_name;

use crate::{error::AppError, Machine};
use strum::Display;
use tap::Tap;
use log::*;

#[derive(Clone)]
pub struct AppTestContext {

}

#[derive(Clone)]
pub enum TestResult {
  Pass(PassData),
  Fail(FailData),
  Inconclusive(InconclusiveData),
}

#[derive(Clone)]
pub struct PassData {
  pub context: MachineTestContext,
  pub test_name: String,
}

#[derive(Clone)]
pub struct FailData {
  pub context: MachineTestContext,
  pub reason: String,
  pub suggestion: String,
  pub test_name: String,
}

#[derive(Clone)]
pub struct InconclusiveData {
  pub context: MachineTestContext,
  pub test_name: String,
  pub action: ConclusiveAction,
}

#[derive(Clone)]
pub enum ConclusiveAction {
  TestRequest(TestRequestData),
  ManualInstruction(String),
}

#[derive(Clone)]
pub struct TestRequestData {
  pub requested_test_name: String,
  pub requesting_test_name: String,
}

impl TestResult {

  fn type_name(&self) -> & 'static str {
    match self {
      TestResult::Pass(_) => "Pass",
      TestResult::Fail(_) => "Fail",
      TestResult::Inconclusive(_) => "TestRequest",
    }
  }


  pub fn default(context: &MachineTestContext, test_name: &str) -> TestResult {
    TestResult::Fail(FailData {
      context: context.clone(),
      reason: "Not implemeneted".into(),
      suggestion: "Not implemented".into(),
      test_name: test_name.into(),
    })
  }

}

impl std::fmt::Display for TestResult {

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.type_name())
  }

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

// #[derive(Clone, Debug, Display, PartialEq)]
// pub enum TestStatus {
//   Pass,
//   Fail,
// }

// impl TestStatus {

//   pub fn from(b: bool) -> TestStatus {
//     if b {
//       TestStatus::Pass
//     } else {
//       TestStatus::Fail
//     }
//       .tap(|x| trace!("{:?} to {:?}", b, x))
//   }

// }
