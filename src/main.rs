// Use the library crate's modules
use nix_remote_builder_doctor::*;

use matching_keys_test::MatchingKeysTest;
use output::{suggestions_print, table_print, json_print};
use partial_application::partial;
use crate::{
  dns_test::DnsTest,
  error::AppError,
  connection_test::ConnectionTest,
  host_key_test::HostKeyTest,
  local_to_remote_build_test::LocalToRemoteBuildTest,
  remote_build_test::RemoteBuildTest,
  test::{
    AppTestContext,
    AppTestResults,
    MachineTestContext,
    MachineTestResult,
    Test,
  },
};
use clap::Parser;
use cli::{Cli, OutputFormat};
use log::*;
use machine::{parse_all, Machine};
use std::fs;

fn test_results(
  app_context: &AppTestContext,
  machines: &Vec<Machine>,
  test_filter: &Vec<String>,
) -> Result<AppTestResults, AppError> {
  Ok(AppTestResults {
    machine_test_results: machines
      .into_iter()
      .map(|m| machine_test_results(app_context, m, test_filter))
      .collect::<Result<Vec<MachineTestResult>, AppError>>()
      ?,
  })
}

fn machine_test_results(
  app_context: &AppTestContext,
  machine: &Machine,
  test_filter: &Vec<String>,
) -> Result<MachineTestResult, AppError> {
  let context = MachineTestContext {
    app_context: app_context.clone(),
    machine: machine.clone(),
  };

  // Define all tests with their kebab-case names
  let all_tests: Vec<(&str, Box<dyn Test>)> = vec![
    ("dns", Box::new(DnsTest {})),
    ("matching-keys", Box::new(MatchingKeysTest {})),
    ("connection", Box::new(ConnectionTest {})),
    ("host-key", Box::new(HostKeyTest {})),
    ("remote-build", Box::new(RemoteBuildTest {})),
    ("local-to-remote-build", Box::new(LocalToRemoteBuildTest {})),
  ];

  // Filter tests based on the provided filter
  let tests_to_run: Vec<Box<dyn Test>> = if test_filter.is_empty() {
    all_tests.into_iter().map(|(_, test)| test).collect()
  } else {
    all_tests
      .into_iter()
      .filter(|(name, _)| test_filter.contains(&name.to_string()))
      .map(|(_, test)| test)
      .collect()
  };

  // Run the filtered tests
  let test_results = tests_to_run
    .into_iter()
    .map(|test| test.test(&context))
    .collect::<Result<Vec<_>, AppError>>()?;

  Ok(MachineTestResult {
    machine: machine.clone(),
    test_results,
  })
}

fn host_exclude(
  excludes: &Vec<String>,
  machines: Vec<Machine>,
) -> Vec<Machine> {
  machines
    .into_iter()
    .filter(move |machine| {
      !excludes
        .into_iter()
        .any(|exclude| {
          machine.url.host_str().unwrap_or("").contains(exclude)
        })
    })
    .collect()
}

fn host_include(
  includes: &Vec<String>,
  machines: Vec<Machine>,
) -> Vec<Machine> {
  machines
    .into_iter()
    .filter(move |machine| {
      includes
        .into_iter()
        .any(|include| {
          machine.url.host_str().unwrap_or("").contains(include)
        })
    })
    .collect()
}

fn main() -> Result<(), AppError> {
  let cli = Cli::parse();
  logger::logger_init(&cli.verbosity)?;
  let machines: Vec<Machine> = fs::read_to_string("/etc/nix/machines")
    .map_err(AppError::MachinesFileReadError)
    .and_then(machine::parse_raw)
    .and_then(|raws| {
      parse_all(raws)
        .into_iter()
        .inspect(|res| {
          match res {
            Ok(m) => trace!("Parse of machine successful: {:?}", m),
            Err(e) => error!("Could not handle entry:\n{:?}", e),
          };
        })
        .filter(|x| {
          x.is_ok()
        })
        .collect::<Result<Vec<Machine>, AppError>>()
    })
    .map(partial!(host_exclude => &cli.exclude, _))
    .map(partial!(host_include => &cli.include, _))
    ?;
  let context = AppTestContext {};
  let results = test_results(&context, &machines, &cli.tests)?.machine_test_results;

  match cli.format {
    cli::OutputFormat::Table => {
      table_print(&results);
      suggestions_print(&results);
    },
    cli::OutputFormat::Json => {
      json_print(&results);
    },
  }

  Ok(())
}
