//! Age helpers for reading and writing age-encrypted data.
//!
//! These were initially written with a particular matching test in
//! mind, but that test was incorrect in its assumptions.  The code
//! still seems potentially useful so we'll keep it around for now.

use crate::{command::command_with_stdin, error::AppError};
use std::path::Path;

pub fn age_encrypt(
  recipients_path: &Path,
  data: &str,
) -> Result<String, AppError> {
  command_with_stdin(
    "rage",
    vec![
      "--encrypt",
      "--output",
      "-",
      "--recipients-file",
      recipients_path
        .to_str()
        .ok_or(AppError::AgeRecipientsPathInvalidUtf8Error)?,
    ],
    data.as_bytes(),
  )
  .and_then(|res| {
    if res.status.success() {
      Ok(String::from_utf8_lossy(&res.stdout).to_string())
    } else {
      Err(AppError::AgeEncryptionFailure(
        String::from_utf8_lossy(&res.stderr).to_string(),
      ))
    }
  })
}

pub fn age_decrypt(
  private_key_path: &Path,
  encrypted_data: &str,
) -> Result<String, AppError> {
  command_with_stdin(
    "rage",
    vec![
      "--decrypt",
      "--identity",
      private_key_path
        .to_str()
        .ok_or(AppError::AgeIdentityPathInvalidUtf8Error)?,
    ],
    encrypted_data.as_bytes(),
  )
  .and_then(|res| {
    if res.status.success() {
      Ok(String::from_utf8_lossy(&res.stdout).to_string())
    } else {
      Err(AppError::AgeDecryptionFailure(
        String::from_utf8_lossy(&res.stderr).to_string(),
      ))
    }
  })
}
