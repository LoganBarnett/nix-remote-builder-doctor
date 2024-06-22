use tabled::{builder::Builder, settings::Style};

use crate::test::{MachineTestResult, TestResult};

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
             result.status.to_string()
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
