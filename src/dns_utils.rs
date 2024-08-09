use crate::error::AppError;
use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use tap::Tap;
use std::process::{Command, Stdio};
use std::os::unix::process::ExitStatusExt;
use nix::sys::signal::Signal;

lazy_static! {
  pub static ref NSLOOKUP_ADDRESS_REGEX: Regex =
    Regex::new(r"(?m)^Address: (.+)$").unwrap();
}

pub fn resolved_host(host: &str) -> Result<String, AppError> {
  let command = "nslookup";
  let result = Command::new(command)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .arg(host)
    .tap(|c| info!("Sending command '{:?}'...", c))
    .output()
    .map_err(AppError::DnsCommandSpawnProcessError)
    ?;
  let stdout = String::from_utf8_lossy(&result.stdout)
      .tap(|out| {
        info!("{} stdout:\n{}", command, out);
      });
  let stderr = String::from_utf8_lossy(&result.stderr)
      .tap(|out| {
        info!("{} stderr:\n{}", command, out);
      });

  if result.status.success() {
    Ok(
      NSLOOKUP_ADDRESS_REGEX
        .captures_iter(&stdout)
        .map(|c| {
          let (_, [value]) = c.extract();
          debug!("Found: {:?}", value);
          value.to_string()
        })
        .collect::<Vec<String>>()
        .join("")
        .tap(|x| info!("Resolved {} to {}.", host, x))
    )
  } else {
    Err(match result.status.code() {
      Some(code) => AppError::DnsCommandFailedError(
        code,
        stderr.to_string(),
      ),
      None => {
        let signal = Signal::try_from(result.status.signal().unwrap()).unwrap();
        // Treat SIGPIPE separately because we can poll for more information on
        // the error potentially.
        match signal {
          Signal::SIGPIPE => AppError::DnsCommandSigPipeError(
            stderr.to_string(),
          ),
           _ => AppError::DnsCommandSignalError(
            signal.as_str().to_string(),
            stderr.to_string(),
          )
        }
      },
    })
  }
}
