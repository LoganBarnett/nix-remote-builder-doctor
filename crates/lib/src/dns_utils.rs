use crate::error::AppError;
use lazy_regex::regex;
use nix::sys::signal::Signal;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use tap::Tap;
use tracing::{debug, info};

pub fn resolved_host(host: &str) -> Result<String, AppError> {
  let command = "nslookup";
  let result = Command::new(command)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .arg(host)
    .tap(|c| info!("Sending command '{:?}'...", c))
    .output()
    .map_err(AppError::DnsCommandSpawnProcessError)?;
  let stdout = String::from_utf8_lossy(&result.stdout)
    .tap(|out| info!("{} stdout:\n{}", command, out));
  let stderr = String::from_utf8_lossy(&result.stderr)
    .tap(|out| info!("{} stderr:\n{}", command, out));

  // The Option-matching nest below mirrors how libc surfaces "exited"
  // vs. "signalled" — clippy's combinator suggestion (map_or → map_or
  // → match) reads worse than the explicit shape here.
  #[allow(clippy::option_if_let_else)]
  if result.status.success() {
    Ok(
      regex!(r"(?m)^Address: (.+)$")
        .captures_iter(&stdout)
        .map(|c| {
          let (_, [value]) = c.extract();
          debug!("Found: {:?}", value);
          value.to_string()
        })
        .collect::<Vec<String>>()
        .join("")
        .tap(|x| info!("Resolved {} to {}.", host, x)),
    )
  } else {
    Err(match result.status.code() {
      Some(code) => AppError::DnsCommandFailedError(code, stderr.to_string()),
      None => match result.status.signal() {
        Some(raw) => match Signal::try_from(raw) {
          // Treat SIGPIPE separately because we can poll for more
          // information on the error potentially.
          Ok(Signal::SIGPIPE) => {
            AppError::DnsCommandSigPipeError(stderr.to_string())
          }
          Ok(signal) => AppError::DnsCommandSignalError(
            signal.as_str().to_string(),
            stderr.to_string(),
          ),
          Err(_) => AppError::DnsCommandUnknownSignalError(raw),
        },
        None => AppError::DnsCommandNoSignalError,
      },
    })
  }
}
