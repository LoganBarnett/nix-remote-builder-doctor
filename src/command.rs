use std::{fs, io::Write, process::{Command, Output, Stdio}};

use log::*;
use tap::Tap;

use crate::AppError;

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
    .map_err(AppError::RageSpawnProcessError)
    ?;
  let mut stdin = command
    .stdin
    .take()
    .ok_or(AppError::RageSpawnProcessStdinError())
    ?;
  // Appease the threading gods.
  let stdin_copy = stdin_data.to_owned();
  let thread = std::thread::spawn(move || {
    stdin.write_all(&stdin_copy).expect("Failed to write to stdin");
    trace!("Wrote:\n{:?}", stdin_copy);
    fs::write("bar.age", stdin_copy).expect("Failed to save to temp file!");
  });
  let output = command
    .wait_with_output()
    .map_err(AppError::RageSpawnProcessError)
    ;
  let _ = thread.join();
  output.inspect(|out| {
    trace!("Command stdout:\n{}", String::from_utf8_lossy(&out.stdout));
    trace!("Command stderr:\n{}", String::from_utf8_lossy(&out.stderr));
  })
}
