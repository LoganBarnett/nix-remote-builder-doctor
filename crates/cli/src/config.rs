use nix_remote_builder_doctor_lib::{LogFormat, LogLevel, OutputFormat};
use rust_template_foundation::MergeConfig;

/// Resolved configuration produced by the `MergeConfig` derive from
/// the CLI args (and any future config-file merge).  Foundation's
/// `#[foundation_main]` macro builds this in advance of calling
/// `main()`.
#[derive(Debug, Clone, MergeConfig)]
#[merge_config(app_name = "nix-remote-builder-doctor")]
pub struct Config {
  #[merge_config(common)]
  pub log_level: LogLevel,
  #[merge_config(common)]
  pub log_format: LogFormat,
  /// Path of the machines file.
  #[merge_config(short, default = "\"/etc/nix/machines\".to_string()")]
  pub machines_file: String,
  /// Exclude hosts whose hostname contains the provided string.
  /// Repeatable for additional excludes.
  #[merge_config(short, default = "Vec::new()")]
  pub exclude: Vec<String>,
  /// Include only hosts whose hostname contains the provided string.
  /// Repeatable for additional includes (OR).  Defaults to a single
  /// empty string so every host matches when no value is set.
  #[merge_config(short, default = "vec![String::new()]")]
  pub include: Vec<String>,
  /// Output format: table or json.
  #[merge_config(short = 'f', default = "OutputFormat::Table")]
  pub format: OutputFormat,
  /// Run only the named tests.  Available: dns, matching-keys,
  /// connection, host-key, remote-build, local-to-remote-build.
  /// Repeatable for additional tests.
  #[merge_config(short = 't', name = "test", default = "Vec::new()")]
  pub tests: Vec<String>,
}
