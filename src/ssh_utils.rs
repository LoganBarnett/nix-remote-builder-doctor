/**
 * These are general SSH utilities.  Not to be confused with establishing and
 * using an SSH connection itself, as is found in ssh.rs.
 *
 * Examples of utility functions:
 * 1. Reading from the local ssh configuration.
 * 2. Doing a key scan on another host.
 * 3. Querying the status of the SSH agent.
 */

use crate::{
  dns_utils::resolved_host,
  error::AppError,
};
use log::*;
use regex::Regex;
use tap::Tap;
use std::process::{Command, Stdio};
use std::os::unix::process::ExitStatusExt;
use nix::sys::signal::Signal;

// For now, until I can figure out the borrowing of a nested loop.
#[derive(Clone)]
pub struct KeyscanEntry {
  pub comment: Option<String>,
  pub key_data: String,
  pub r#type: String,
}

impl KeyscanEntry {

  pub fn parse(s: &str) -> Result<KeyscanEntry, AppError> {
    let segments = s
      .trim()
      .split(" ")
      .into_iter()
      .collect::<Vec<&str>>();
    Ok(KeyscanEntry {
      r#type: segments
        .get(0)
        .ok_or(AppError::SshKeyscanParseError(format!(
          "key_type missing for {}.",
          s,
        )))?
        .to_string(),
      key_data: segments
        .get(1)
        .ok_or(AppError::SshKeyscanParseError(format!(
          "key_data missing for {}.",
          s,
        )))?
        .to_string(),
      comment: segments
        .get(2)
        .map(|x| x.to_string()),
    })
  }

}

impl ToString for KeyscanEntry {
  fn to_string(&self) -> String {
    format!(
      "{} {}{}",
      self.r#type,
      self.key_data,
      self
        .comment
        // Not really sure why this is required since this is all immutable.
        .clone()
        .map(|x| format!(" {}", x))
        .unwrap_or("".into()),
    )
  }
}


/**
 * Run ssh-keyscan against the host and retrieve the keys.
 *
 * It should be noted that ssh-keyscan can fail in a surprising way: During the
 * banner exchange the remote sshd can disconnect because pre-auth fails.
 * ssh-keyscan fails with a SIGPIPE as a result.  This is because the host key
 * algorithms aren't supported by the sshd instance.  If that's the only issue,
 * we can SSH to the host and compare the keys to provide a better report.  That
 * is why we have a separate SshKeyscanCommandSigPipeError.
 */
pub fn ssh_keyscan(
  host: &str,
  port: u16,
) -> Result<Vec<KeyscanEntry>, AppError> {
  // This is fun.
  // https://answers.launchpad.net/debian/+source/openssh/1:9.1p1-1 mentions a
  // problem with ssh-keyscan where a one-byte overflow from the SSH banner on
  // the destination host will cause some kind of problem with ssh-keyscan.
  // This is fixed in 1:9.1p1-1.  The macOS OpenSSH is OpenSSH_9.6p1, LibreSSL
  // 3.3.6.  I assume that means there's already a fix in place for macOS then,
  // but then why are we seeing this error?

  // Running with -vvv results in us seeing the banner not being processed.
  //
  // Okay more progress.  I can run this ktrace (a macOS equivalent of strace):
  // sudo ktrace trace -S -f C3 -c  ssh-keyscan -p 31022 127.0.0.1
  // I should record this in my notes!  Here's the page where I found it:
  // https://stackoverflow.com/a/76987834
  //
  // I can see this error:
  // write (127.0.0.1): Broken pipe
  // # 127.0.0.1:31022 SSH-2.0-OpenSSH_9.7
  // write (127.0.0.1): Broken pipe
  // write (127.0.0.1): Broken pipe
  // # 127.0.0.1:31022 SSH-2.0-OpenSSH_9.7
  // [127.0.0.1]:31022 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJBWcxb/Blaqt1auOtE+F8QUWrUotiC5qBJ+UuEWdVCb
  // Which is funny, because it looks like it's finding the host, and printing
  // the key like it should.  But this broken pipe is to a file descriptor where
  // the host has already disconnected I guess.  ssh-keyscan is not handling
  // this error correctly.
  //
  // I can use this to watch the logs of sshd on the VM that I'm trying to
  // connect to:
  //
  // Unable to negotiate with 10.0.2.2 port 52515: no matching host key type found. Their offer: ecdsa-sha2-nistp256,ecdsa-sha2-nistp384,ecdsa-sha2-nistp521 [preauth]
  // Connection closed by 10.0.2.2 port 52516 [preauth]
  //
  // Line wrapped for readability:
  // Unable to negotiate with 10.0.2.2 port 52515: no matching host key type
  // found. Their offer:
  // ecdsa-sha2-nistp256,ecdsa-sha2-nistp384,ecdsa-sha2-nistp521 [preauth]
  // Connection closed by 10.0.2.2 port 52516 [preauth]
  //
  // So the host key type needs to match...?  But I'm not sure what that means
  // here.  More investigation is needed.  Once I move past this, I should
  // probably try to file a proper bug.
  //
  // This may be a promising to pull:
  // https://askubuntu.com/questions/268272/ssh-refusing-connection-with-message-no-hostkey-alg
  let result = Command::new("ssh-keyscan")
    // These prevent ssh-keyscan from failing with SIGPIPE.
    // Lies!
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    // .stdout(Stdio::inherit())
    // .stderr(Stdio::inherit())
    // The host argument must come last, so port comes first.
    .arg("-p")
    .arg(port.to_string())
    // Why do we need to specify a type?  Shouldn't the absence of a type imply
    // all types?
    .arg("-t")
    .arg("ed25519")
    .arg("-vvv")
    // ssh-keyscan can result in a SIGPIPE error if it is provided something it
    // doesn't know how to resolve for the hostname.  In the cases observed,
    // it's been "localhost".  If 127.0.0.1 is provided, everything works fine.
    .arg(resolved_host(host)?)
    .tap(|c| info!("Sending command '{:?}'...", c))
    .output()
    .map_err(AppError::SshKeyscanCommandSpawnProcessError)
    ?;
  let stdout = String::from_utf8_lossy(&result.stdout)
      .tap(|out| {
        info!("ssh-keyscan stdout:\n{}", out);
      });
  let stderr = String::from_utf8_lossy(&result.stderr)
      .tap(|out| {
        info!("ssh-keyscan stderr:\n{}", out);
      });
  if result.status.success() {
    stdout
      .split("\n")
      .into_iter()
      // Skip the empty line.
      // TODO: Do this without copying the string.
      .filter(|x| x.to_string() != "")
      .map(|x| {
        KeyscanEntry::parse(
          // The first segment from ssh-keyscan will be the host and port in the
          // form of:
          // [127.0.0.1]:17022
          // or:
          // 127.0.0.1
          // We need to skip it.
          &x
            .split(" ")
            .into_iter()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ")
            .tap(|x| debug!("Parsing keyscan entry '{}'...", x))
        )
      })
      .collect::<Result<Vec<KeyscanEntry>, AppError>>()
  } else {
    Err(match result.status.code() {
      Some(code) => AppError::SshKeyscanCommandFailedError(
        code,
        stderr.to_string(),
      ),
      None => {
        let signal = Signal::try_from(result.status.signal().unwrap()).unwrap();
        // Treat SIGPIPE separately because we can poll for more information on
        // the error potentially.
        match signal {
          Signal::SIGPIPE => AppError::SshKeyscanCommandSigPipeError(
            stderr.to_string(),
          ),
           _ => AppError::SshKeyscanCommandSignalError(
            signal.as_str().to_string(),
            stderr.to_string(),
          )
        }
      },
    })
  }
}

pub fn ssh_config_value(
  field: &str,
  hostname: &str,
) -> Result<String, AppError> {
  let result = Command::new("ssh")
    .arg("-G")
    .arg(hostname)
    .output()
    .map_err(AppError::SshSpawnProcessError)
    ?;
  if result.status.success() {
    let regex = Regex::new(format!("^{} (.+?)$", field).as_str()).unwrap();
    String::from_utf8_lossy(&result.stdout)
      .split("\n")
      .into_iter()
      .map(|s: &str| {
        // debug!("Line from ssh config: {:?}", s);
        regex
          .captures_iter(s)
          .map(|c| {
            let (_, [value]) = c.extract();
            debug!("{} found: {:?}", field, value);
            value.to_string()
          })
      })
    // This is very much _magic_.  The list of Options is coerced into a
    // list of values with the Nones removed.  There is some documentation
    // to that effect but it is difficult to search for:
    // https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.flatten
      .flatten()
      .collect::<Vec<_>>()
      .get(0)
      .ok_or(AppError::SshConfigQueryFieldMissingError(
        hostname.to_string(),
        field.to_string(),
      ))
      .cloned()
  } else {
    Err(AppError::SshConfigQueryError(hostname.to_string()))
  }
}
