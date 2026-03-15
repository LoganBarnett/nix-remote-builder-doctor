// Library root for nix-remote-builder-doctor
// This file allows us to organize code as a library for testing purposes

pub mod age;
pub mod cli;
pub mod command;
pub mod connection_test;
pub mod dns_test;
pub mod dns_utils;
pub mod error;
pub mod host_key_test;
pub mod local_to_remote_build_test;
pub mod logger;
pub mod machine;
pub mod matching_keys_test;
pub mod nix_store_permission_test;
pub mod output;
pub mod remote_build_test;
pub mod ssh;
pub mod ssh_utils;
pub mod ssh2_adapter;
pub mod test;
pub mod test_helpers;

// Re-export commonly used types
pub use cli::Cli;
pub use error::AppError;
pub use machine::Machine;
pub use test::{AppTestContext, MachineTestContext, MachineTestResult, TestResult};