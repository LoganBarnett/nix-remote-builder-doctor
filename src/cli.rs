use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
  Table,
  Json,
}

#[derive(Parser)]
#[command(
  name = "nix-remote-builder-doctor",
  about = "Diagnose Nix remote builder issues.",
)]
pub struct Cli {
  #[arg(
    env,
    short,
    long,
    default_value = "/etc/nix/machines",
    help = "Specify the path of the machines file.",
  )]
  pub machines_file: String,
  #[arg(
    env,
    short,
    long,
    help = "Exclude hosts containing the provided string.  Specify multiple times for additional excludes.",
  )]
  pub exclude: Vec<String>,
  #[arg(
    // Setting the default to a single empty string means we see if any match
    // includes an empty string.  Every match will, so everything is included by
    // default.
    default_values_t = vec!("".to_string()),
    env,
    short,
    long,
    help = "Include only hosts containing the provided string.  Specify multiple times for additional includes (OR).",
  )]
  pub include: Vec<String>,
  #[arg(
    short = 'f',
    long,
    value_enum,
    default_value = "table",
    help = "Output format (table or json).",
  )]
  pub format: OutputFormat,
  #[arg(
    short = 't',
    long = "test",
    help = "Run only specific tests. Available tests: dns, matching-keys, connection, host-key, remote-build, local-to-remote-build. Specify multiple times for additional tests.",
  )]
  pub tests: Vec<String>,
  #[command(flatten)]
  pub verbosity: clap_verbosity_flag::Verbosity,
}
