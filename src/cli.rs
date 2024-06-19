#[derive(Clap)]
#[clap(
  name = "nix-remote-builder-doctor",
  about = "Diagnose Nix remote builder issues.",
)]
#[clap(setting = clap::AppSettings::ColoredHelp)]
pub struct Cli {
  pub machines_file: Path,
  pub exclude: Vec<String>,
  pub verbosity: usize,
}
