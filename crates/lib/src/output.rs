use clap::ValueEnum;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::{builder::Builder, settings::Style};

use crate::error::AppError;
use crate::test::{ConclusiveAction, MachineTestResult, TestResult};

/// The format `nix-remote-builder-doctor` writes results in.
///
/// Lives in `output` because the variants line up one-to-one with the
/// `*_print` functions below.  Derives `clap::ValueEnum` and `serde`
/// traits so the foundation `MergeConfig` derive can accept it on a
/// CLI flag and (eventually) a config-file field.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
  Table,
  Json,
}

// pub struct TestResultTableRecord {
//   pub hostname: String,
//   pub test_results: Vec<TestResult>,
// }

pub fn table_print(records: &[MachineTestResult]) {
  let header = vec![
    "Host".to_string(),
    // Add new tests after the Host entry above.
    "DNS".to_string(),
    "Matching Keys".to_string(),
    "Connection".to_string(),
    "Host Key".to_string(),
    "Remote Build".to_string(),
    "Local To Remote Build".to_string(),
  ];
  let rows: Vec<Vec<String>> = std::iter::once(header)
    .chain(records.iter().map(|r| {
      let mut row = vec![r
        .machine
        .url
        .host_str()
        .unwrap_or("unknown host")
        .to_string()];
      row.extend(r.test_results.iter().map(|result| match result {
        TestResult::Pass(_) => result.to_string().green().to_string(),
        TestResult::Fail(_) => result.to_string().red().to_string(),
        TestResult::Inconclusive(_) => result.to_string().yellow().to_string(),
      }));
      row
    }))
    .collect();
  let table = Builder::from(rows)
    .build()
    .with(Style::rounded())
    .to_string();
  println!("{}", table);
}

pub fn suggestions_print(records: &[MachineTestResult]) {
  let output = records
    .iter()
    .flat_map(|record| {
      record.test_results.iter().map(move |result| {
        // Probably no type refinement available in Rust.  Would be nice
        // to know for sure.
        match result {
          TestResult::Inconclusive(data) => match &data.action {
            ConclusiveAction::ManualInstruction(instruction) => {
              instruction.clone()
            }
            ConclusiveAction::TestRequest(data) => {
              format!("See {}", data.requested_test_name)
            }
          },
          TestResult::Pass(_) => String::new(),
          TestResult::Fail(data) => format!(
            "Test {} for {} has failed.\n  Reason: {}\n  Suggestion: {}",
            data.test_name,
            record.machine.url.host_str().unwrap_or("unknown host"),
            data.reason,
            data.suggestion,
          ),
        }
      })
    })
    .collect::<Vec<String>>()
    .join("\n");
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

pub fn json_print(records: &[MachineTestResult]) -> Result<(), AppError> {
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
      let hostname = record
        .machine
        .url
        .host_str()
        .unwrap_or("unknown")
        .to_string();
      let mut first_failure = None;
      let mut overall_healthy = true;

      let checks: Vec<JsonCheck> = record
        .test_results
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
            }
            TestResult::Inconclusive(data) => {
              let msg = match &data.action {
                ConclusiveAction::ManualInstruction(instruction) => {
                  instruction.clone()
                }
                ConclusiveAction::TestRequest(req_data) => {
                  format!(
                    "Skipped - depends on {}",
                    req_data.requested_test_name
                  )
                }
              };
              ("skip".to_string(), Some(msg), None, None)
            }
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
        overall_status: if overall_healthy {
          "healthy".to_string()
        } else {
          "unhealthy".to_string()
        },
        first_failure,
      }
    })
    .collect();

  let healthy_count = builders
    .iter()
    .filter(|b| b.overall_status == "healthy")
    .count();
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

  println!(
    "{}",
    serde_json::to_string_pretty(&output)
      .map_err(AppError::JsonSerializationError)?
  );
  Ok(())
}
