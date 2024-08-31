use clap::Parser;

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
  #[command(flatten)]
  pub verbosity: clap_verbosity_flag::Verbosity,
}
