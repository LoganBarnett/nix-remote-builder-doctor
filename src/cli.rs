use clap::Parser;

#[derive(Parser)]
#[command(
  name = "nix-remote-builder-doctor",
  about = "Diagnose Nix remote builder issues.",
)]
pub struct Cli {
  #[arg(env, short, long, default_value = "/etc/nix/machines")]
  pub machines_file: String,
  #[arg(env, short, long)]
  pub exclude: Vec<String>,
  #[command(flatten)]
  pub verbosity: clap_verbosity_flag::Verbosity,
}
