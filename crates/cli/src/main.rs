//! nix-remote-builder-doctor — entry point.
//!
//! The `#[foundation_main]` macro handles CLI parsing, config
//! resolution, and tracing init.  This file just delegates into the
//! library crate's `run` function with the resolved config.

mod config;

use config::Config;
use nix_remote_builder_doctor_lib::{run, AppError, RunArgs};
use rust_template_foundation::main as foundation_main;
use std::process::ExitCode;

#[foundation_main]
pub fn main(config: Config) -> Result<ExitCode, AppError> {
  run(&RunArgs {
    machines_file: &config.machines_file,
    exclude: &config.exclude,
    include: &config.include,
    format: config.format,
    tests: &config.tests,
  })?;
  Ok(ExitCode::SUCCESS)
}
