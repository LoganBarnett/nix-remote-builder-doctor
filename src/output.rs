use owo_colors::OwoColorize;
use tabled::{builder::Builder, settings::Style};


use crate::test::{ConclusiveAction, MachineTestResult, TestResult};

// pub struct TestResultTableRecord {
//   pub hostname: String,
//   pub test_results: Vec<TestResult>,
// }


pub fn table_print(records: &Vec<MachineTestResult>) -> () {
  let rows = vec!(
    vec!(vec!(
      "Host".into(),
      // Add new tests after the Host entry above.
      "DNS".into(),
      "Matching Keys".into(),
      "Connection".into(),
      "Remote Build".into(),
      "Local To Remote Build".into(),
    )),
    records
      .into_iter()
      .map(move |r| {
        vec!(
          vec!(r.machine.url.host_str().unwrap_or("unknown host").to_string()),
          r.test_results
           .iter()
           .map(|result| {
             match result {
               TestResult::Pass(_) => result.to_string().green().to_string(),
               TestResult::Fail(_) => result.to_string().red().to_string(),
               TestResult::Inconclusive(_) => result.to_string().yellow().to_string(),
             }
           })
           .collect::<Vec<String>>(),
        ).concat()
      })
      .collect::<Vec<Vec<String>>>(),
  ).concat();
  let builder = Builder::from(rows);
  let table = builder
    .build()
    .with(Style::rounded())
    .to_string();
  println!("{}", table);
}

pub fn suggestions_print(records: &Vec<MachineTestResult>) -> () {
  let output = records
    .into_iter()
    .map(|record| {
      (&record.test_results)
        .into_iter()
        .map(|result| {
          // Probably no type refinement available in Rust.  Would be nice to
          // know for sure.
          match result {
            TestResult::Inconclusive(data) => {
              match &data.action {
                ConclusiveAction::ManualInstruction(instruction) => {
                  instruction.clone()
                },
                ConclusiveAction::TestRequest(data) => {
                  format!("See {}", data.requested_test_name)
                },
              }
            },
            TestResult::Pass(_data) => "".into(),
            TestResult::Fail(data) => format!(
              "Test {} for {} has failed.\n  Reason: {}\n  Suggestion: {}",
              data.test_name,
              record.machine.url.host_str().unwrap_or("unknown host"),
              data.reason,
              data.suggestion,
            ).to_string(),
          }
        })
        .collect::<Vec<String>>()
    })
    .flatten()
    .collect::<Vec<String>>()
    .join("\n")
    ;
  println!("{}", output);
}
