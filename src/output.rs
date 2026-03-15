use owo_colors::OwoColorize;
use serde::{Serialize, Deserialize};
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
      "Host Key".into(),
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

// JSON output structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
  pub builders: Vec<JsonBuilder>,
  pub summary: JsonSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonBuilder {
  pub hostname: String,
  pub checks: Vec<JsonCheck>,
  pub overall_status: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub first_failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonCheck {
  pub name: String,
  pub status: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub duration_ms: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reason: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSummary {
  pub total_builders: usize,
  pub healthy: usize,
  pub unhealthy: usize,
  pub overall: String,
}

pub fn json_print(records: &Vec<MachineTestResult>) -> () {
  let test_names = vec![
    ("DNS", "dns"),
    ("Matching Keys", "matching-keys"),
    ("Connection", "connection"),
    ("Host Key", "host-key"),
    ("Remote Build", "remote-build"),
    ("Local To Remote Build", "local-to-remote-build"),
  ];

  let builders: Vec<JsonBuilder> = records
    .iter()
    .map(|record| {
      let hostname = record.machine.url.host_str().unwrap_or("unknown").to_string();
      let mut first_failure = None;
      let mut overall_healthy = true;

      let checks: Vec<JsonCheck> = record.test_results
        .iter()
        .zip(&test_names)
        .map(|(result, (display_name, _kebab_name))| {
          let (status, message, reason, suggestion) = match result {
            TestResult::Pass(_) => ("pass".to_string(), None, None, None),
            TestResult::Fail(data) => {
              if first_failure.is_none() {
                first_failure = Some(display_name.to_string());
              }
              overall_healthy = false;
              (
                "fail".to_string(),
                Some(format!("{} test failed", display_name)),
                Some(data.reason.clone()),
                Some(data.suggestion.clone()),
              )
            },
            TestResult::Inconclusive(data) => {
              let msg = match &data.action {
                ConclusiveAction::ManualInstruction(instruction) => instruction.clone(),
                ConclusiveAction::TestRequest(req_data) => {
                  format!("Skipped - depends on {}", req_data.requested_test_name)
                },
              };
              ("skip".to_string(), Some(msg), None, None)
            },
          };

          JsonCheck {
            name: display_name.to_string(),
            status,
            message,
            duration_ms: None, // Could be added later if timing is implemented
            reason,
            suggestion,
          }
        })
        .collect();

      JsonBuilder {
        hostname,
        checks,
        overall_status: if overall_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        first_failure,
      }
    })
    .collect();

  let healthy_count = builders.iter().filter(|b| b.overall_status == "healthy").count();
  let unhealthy_count = builders.len() - healthy_count;

  let summary = JsonSummary {
    total_builders: builders.len(),
    healthy: healthy_count,
    unhealthy: unhealthy_count,
    overall: if unhealthy_count == 0 {
      "healthy".to_string()
    } else if healthy_count == 0 {
      "unhealthy".to_string()
    } else {
      "degraded".to_string()
    },
  };

  let output = JsonOutput { builders, summary };

  println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
