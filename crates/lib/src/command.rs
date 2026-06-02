use std::{
  io::Write,
  process::{Command, Output, Stdio},
};

use tap::Tap;
use tracing::{debug, trace};

use crate::AppError;

/// Spawn a child process, pipe `stdin_data` to its stdin, and return
/// its `Output`.  The single caller today is the `age` helper feeding
/// data into `rage`; if a non-rage caller appears, generalise the
/// error variants and rename to match.
pub fn command_with_stdin(
  exec: &str,
  args: Vec<&str>,
  stdin_data: &[u8],
) -> Result<Output, AppError> {
  let mut command = Command::new(exec)
    .args(args)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .tap(|x| debug!("++ Begin command ++:\n{:?}\n++ End Command ++", x))
    .spawn()
    .map_err(AppError::RageSpawnProcessError)?;
  // Write stdin inline.  An earlier revision did this on a background
  // thread plus an `expect`-on-write; the threading existed because
  // rage's stdout could fill before stdin drained, but rage encrypts
  // the inputs we feed it (a few hundred bytes at most), well below
  // any pipe-buffer threshold.  Inline writes let errors propagate
  // through `?` instead of needing a JoinHandle / Result channel.
  {
    let mut stdin = command
      .stdin
      .take()
      .ok_or(AppError::RageSpawnProcessStdinError)?;
    stdin
      .write_all(stdin_data)
      .map_err(AppError::RageStdinWriteError)?;
    trace!("Wrote:\n{:?}", stdin_data);
  }
  command
    .wait_with_output()
    .map_err(AppError::RageSpawnProcessError)
    .inspect(|out| {
      trace!("Command stdout:\n{}", String::from_utf8_lossy(&out.stdout));
      trace!("Command stderr:\n{}", String::from_utf8_lossy(&out.stderr));
    })
}
