use owo_colors::OwoColorize;
use tabled::{builder::Builder, settings::Style};


use crate::test::{MachineTestResult, TestResult};

// pub struct TestResultTableRecord {
//   pub hostname: String,
//   pub test_results: Vec<TestResult>,
// }


pub fn table_print(records: &Vec<MachineTestResult>) -> () {
  let rows = vec!(
    vec!(vec!(
      "Host".into(),
      "Connection".into(),
      "Matching Keys".into(),
      "Remote Build".into(),
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
            TestResult::Pass(data) => "".into(),
            TestResult::Fail(data) => format!(
              "Test {} is {} for test failed.\n  Reason: {}\n  Suggestion: {}",
              record.machine.url.host_str().unwrap_or("unknown host"),
              data.test_name,
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
