mod test;
mod machine;
mod error;
mod nix_store_permission_test;
mod ssh;
mod connection_test;
mod local_to_remote_build_test;
mod ssh2_adapter;
mod remote_build_test;
// mod russh;

use crate::test::Test;
pub(crate) use crate::error::AppError;
use connection_test::ConnectionTest;
use machine::{parse_all, Machine};
use remote_build_test::RemoteBuildTest;
use test::{AppTestContext, AppTestResults, MachineTestContext, MachineTestResults};
use std::fs;

fn test_results(
  app_context: &AppTestContext,
  machines: &Vec<Machine>,
) -> Result<AppTestResults, AppError> {
  Ok(AppTestResults {
    machine_test_results: machines
      .into_iter()
      .map(|m| machine_test_results(app_context, m))
      .collect::<Result<Vec<MachineTestResults>, AppError>>()
      ?,
  })
}

fn machine_test_results(
  app_context: &AppTestContext,
  machine: &Machine,
) -> Result<MachineTestResults, AppError> {
  let context = MachineTestContext {
    app_context: app_context.clone(),
    machine: machine.clone(),
  };
  Ok(MachineTestResults {
    machine: machine.clone(),
    test_results: vec!(
      // ConnectionTest {}.test(&context)?,
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
          if x.pass {
            "pass".to_string()
          } else {
            format!(
              "fail: {:?}\nssh -o \"IdentitiesOnly=yes\" -i {} {}",
              x.reason,
              x.context.machine.private_key_path,
              x.context.machine.url.to_string(),
            )
          }
        })
        .collect::<Vec<String>>()
        .join("\n")
        ,
    );
  }
}

fn main() -> Result<(), AppError> {
  let machines: Vec<Machine> = fs::read_to_string("/etc/nix/machines")
    .map_err(AppError::MachinesFileReadError)
    .and_then(machine::parse_raw)
    .and_then(parse_all)
    ?;
  let context = AppTestContext {};
  output(&test_results(&context, &machines)?);
  Ok(())
}
