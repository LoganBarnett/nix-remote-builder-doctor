mod test;
mod machine;
mod error;
mod nix_store_permission_test;
mod ssh;
mod connection_test;
mod local_to_remote_build_test;
mod ssh2_adapter;
mod remote_build_test;
mod cli;
mod output;
mod matching_keys_test;
mod logger;
mod command;

use matching_keys_test::MatchingKeysTest;
use output::{suggestions_print, table_print};
use partial_application::partial;
use crate::test::{Test, TestStatus};
pub(crate) use crate::error::AppError;
use clap::Parser;
use cli::Cli;
use connection_test::ConnectionTest;
use log::*;
use machine::{parse_all, Machine};
use remote_build_test::RemoteBuildTest;
use test::{AppTestContext, AppTestResults, MachineTestContext, MachineTestResult};
use std::fs;

fn test_results(
  app_context: &AppTestContext,
  machines: &Vec<Machine>,
) -> Result<AppTestResults, AppError> {
  Ok(AppTestResults {
    machine_test_results: machines
      .into_iter()
      .map(|m| machine_test_results(app_context, m))
      .collect::<Result<Vec<MachineTestResult>, AppError>>()
      ?,
  })
}

fn machine_test_results(
  app_context: &AppTestContext,
  machine: &Machine,
) -> Result<MachineTestResult, AppError> {
  let context = MachineTestContext {
    app_context: app_context.clone(),
    machine: machine.clone(),
  };
  Ok(MachineTestResult {
    machine: machine.clone(),
    test_results: vec!(
      MatchingKeysTest {}.test(&context)?,
      ConnectionTest {}.test(&context)?,
      RemoteBuildTest {}.test(&context)?,
    ),
  })
}

fn output(results: &AppTestResults) -> () {
  for test_result in &results.machine_test_results {
    println!(
      "{}: {}",
      test_result.machine.url,
      test_result
        .test_results
        .clone()
        .into_iter()
        .map(|x| {
          match x.status {
            TestStatus::Pass => "pass".to_string(),
            TestStatus::Fail => {
              format!(
                "fail: {:?}\nssh -o \"IdentitiesOnly=yes\" -i {} {}",
                x.reason,
                x.context.machine.private_key_path,
                x.context.machine.url.to_string(),
              )
            }
          }
        })
        .collect::<Vec<String>>()
        .join("\n")
        ,
    );
  }
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
    ?;
  let context = AppTestContext {};
  // output(&test_results(&context, &machines)?);
  let results = test_results(&context, &machines)?.machine_test_results;
  table_print(&results);
  suggestions_print(&results);
  Ok(())
}
