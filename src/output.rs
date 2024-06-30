use owo_colors::OwoColorize;
use tabled::{builder::Builder, settings::Style};


use crate::test::{MachineTestResult, TestResult, TestStatus::{self, Fail, Pass}};

// pub struct TestResultTableRecord {
//   pub hostname: String,
//   pub test_results: Vec<TestResult>,
// }


pub fn table_print(records: &Vec<MachineTestResult>) -> () {
  let rows = vec!(
    vec!(vec!(
      "Host".to_string(),
      "Connection".to_string(),
      "Remote Build".to_string(),
    )),
    records
      .into_iter()
      .map(move |r| {
        vec!(
          vec!(r.machine.url.host_str().unwrap_or("unknown host").to_string()),
          r.test_results
           .iter()
           .map(|result| {
             match result.status {
               Pass => result.status.to_string().green().to_string(),
               Fail => result.status.to_string().red().to_string(),
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
    .filter(|record| {
      (&record.test_results).into_iter().any(|result| {
        result.status == TestStatus::Fail
      })
    })
    .map(|record| {
      (&record.test_results).into_iter().map(|result| {
        format!(
          "{} failed test {}\n  Reason: {}\n  Suggestion: {}",
          record.machine.url.host_str().unwrap_or("unknown host"),
          result.test_name,
          result.reason,
          result.suggestion,
        )
      })
      .collect::<Vec<String>>()
    })
    .flatten()
    .collect::<Vec<String>>()
    .join("\n")
    ;
  println!("{}", output);
}
