//! Library root for `nix-remote-builder-doctor`.
//!
//! The bulk of the program lives here so it is exercised by unit and
//! integration tests; the CLI crate is a thin entry point that wires
//! `run` up to the foundation-managed CLI/config lifecycle.

pub mod age;
pub mod command;
pub mod connection_test;
pub mod dns_test;
pub mod dns_utils;
pub mod error;
pub mod host_key_test;
pub mod local_to_remote_build_test;
pub mod machine;
pub mod matching_keys_test;
pub mod nix_store_permission_test;
pub mod output;
pub mod remote_build_test;
pub mod ssh;
pub mod ssh2_adapter;
pub mod ssh_utils;
pub mod test;

#[cfg(test)]
pub mod test_helpers;

#[cfg(test)]
mod output_tests;

// Re-export foundation's logging types so the CLI crate's Config can
// reach them through one canonical path.
pub use rust_template_foundation::logging::{LogFormat, LogLevel};
pub use rust_template_foundation::prelude::*;

pub use error::AppError;
pub use machine::Machine;
pub use output::OutputFormat;
pub use test::{
  AppTestContext, MachineTestContext, MachineTestResult, TestResult,
};

use crate::connection_test::ConnectionTest;
use crate::dns_test::DnsTest;
use crate::host_key_test::HostKeyTest;
use crate::local_to_remote_build_test::LocalToRemoteBuildTest;
use crate::matching_keys_test::MatchingKeysTest;
use crate::output::{json_print, suggestions_print, table_print};
use crate::remote_build_test::RemoteBuildTest;
use crate::test::Test;
use partial_application::partial;
use std::fs;
use tracing::{error, trace};

/// Arguments to `run`.  Borrowed slices/strings so the CLI crate can
/// pass them straight from its `Config` without cloning.
pub struct RunArgs<'a> {
  pub machines_file: &'a str,
  pub exclude: &'a [String],
  pub include: &'a [String],
  pub format: OutputFormat,
  pub tests: &'a [String],
}

pub fn run(args: &RunArgs<'_>) -> Result<(), AppError> {
  // Per-machine parse failures are reported via tracing here and the
  // machine is dropped — the doctor proceeds with the well-formed
  // entries.  This is deliberate partial-success behaviour: a single
  // malformed /etc/nix/machines line should not blank the whole
  // report.  The chain reads as filter_map(Result::ok) rather than
  // `.filter(is_ok).collect::<Result>()` so the "logged then
  // discarded" intent is explicit and there is no fake `Result`
  // round-trip.
  let machines: Vec<Machine> = fs::read_to_string(args.machines_file)
    .map_err(AppError::MachinesFileReadError)
    .and_then(machine::parse_raw)
    .map(|raws| {
      machine::parse_all(raws)
        .into_iter()
        .inspect(|res| match res {
          Ok(m) => trace!("Parse of machine successful: {:?}", m),
          Err(e) => error!("Could not handle entry:\n{:?}", e),
        })
        .filter_map(Result::ok)
        .collect()
    })
    .map(partial!(host_exclude => args.exclude, _))
    .map(partial!(host_include => args.include, _))?;
  let context = AppTestContext {};
  let results =
    test_results(&context, &machines, args.tests)?.machine_test_results;

  match args.format {
    OutputFormat::Table => {
      table_print(&results);
      suggestions_print(&results);
    }
    OutputFormat::Json => {
      json_print(&results)?;
    }
  }

  Ok(())
}

fn test_results(
  app_context: &AppTestContext,
  machines: &[Machine],
  test_filter: &[String],
) -> Result<test::AppTestResults, AppError> {
  Ok(test::AppTestResults {
    machine_test_results: machines
      .iter()
      .map(|m| machine_test_results(app_context, m, test_filter))
      .collect::<Result<Vec<MachineTestResult>, AppError>>()?,
  })
}

fn machine_test_results(
  app_context: &AppTestContext,
  machine: &Machine,
  test_filter: &[String],
) -> Result<MachineTestResult, AppError> {
  let context = MachineTestContext {
    app_context: app_context.clone(),
    machine: machine.clone(),
  };

  let all_tests: Vec<(&str, Box<dyn Test>)> = vec![
    ("dns", Box::new(DnsTest {})),
    ("matching-keys", Box::new(MatchingKeysTest {})),
    ("connection", Box::new(ConnectionTest {})),
    ("host-key", Box::new(HostKeyTest {})),
    ("remote-build", Box::new(RemoteBuildTest {})),
    ("local-to-remote-build", Box::new(LocalToRemoteBuildTest {})),
  ];

  let tests_to_run: Vec<Box<dyn Test>> = if test_filter.is_empty() {
    all_tests.into_iter().map(|(_, test)| test).collect()
  } else {
    all_tests
      .into_iter()
      .filter(|(name, _)| test_filter.iter().any(|t| t == name))
      .map(|(_, test)| test)
      .collect()
  };

  let test_results = tests_to_run
    .into_iter()
    .map(|test| test.test(&context))
    .collect::<Result<Vec<_>, AppError>>()?;

  Ok(MachineTestResult {
    machine: machine.clone(),
    test_results,
  })
}

fn host_exclude(excludes: &[String], machines: Vec<Machine>) -> Vec<Machine> {
  machines
    .into_iter()
    .filter(|machine| {
      !excludes
        .iter()
        .any(|exclude| machine.url.host_str().unwrap_or("").contains(exclude))
    })
    .collect()
}

fn host_include(includes: &[String], machines: Vec<Machine>) -> Vec<Machine> {
  machines
    .into_iter()
    .filter(|machine| {
      includes
        .iter()
        .any(|include| machine.url.host_str().unwrap_or("").contains(include))
    })
    .collect()
}
