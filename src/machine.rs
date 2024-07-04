use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::debug;
use url::Url;
use std::fs;
use std::process::Command;

use regex::Regex;
use crate::error::AppError;

pub struct EtcNixMachineRaw {
  pub url: String,
  pub platforms: Vec<String>,
  pub private_key_path: String,
  pub public_key_base64: String,
  pub max_jobs: u32,
  pub speed_factor: u32,
  pub supported_features: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Machine {
  pub url: Url,
  pub platforms: Vec<String>,
  pub private_key: String,
  pub private_key_path: String,
  pub public_key: String,
  pub max_jobs: u32,
  pub speed_factor: u32,
  pub supported_features: Vec<String>,
}

impl Machine {

  pub fn ssh_invocation(&self) -> String {
    format!(
      "sudo ssh -o \"IdentitiesOnly=yes\" -o \"StrictHostKeyChecking=no\" -i {} {}",
      self.private_key_path,
      self.url.to_string(),
    )
  }

}

fn parse_field_string(
  field_name: String,
  field_candidate: Option<&String>,
) -> Result<String, AppError> {
  field_candidate
    .ok_or(AppError::MachinesEntryMissingFieldError(
      field_name.to_string(),
    ))
    .map(|x| x.to_string())
}

fn parse_field_u32(
  field_name: String,
  field_candidate: Option<&String>,
) -> Result<u32, AppError> {
  parse_field_string(
    field_name.clone(),
    field_candidate,
  )
    .and_then({move |x|
      x
        .parse()
        .map_err(|_e| AppError::MachinesEntryNotNumberFieldError(field_name))
    })

}

fn parse_field_vec_string(
  separator: String,
  field_name: String,
  field_candidate: Option<&String>,
) -> Result<Vec<String>, AppError> {
  parse_field_string(
    field_name,
    field_candidate,
  )
    .map(|x| {
      x
        .split(&separator)
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
    })
}

pub fn line_to_machine_raw(line: &str) -> Result<EtcNixMachineRaw, AppError> {
  let parts: Vec<String> = line
    .split(" - ")
    .map(|x| x.to_string())
    .collect::<Vec<String>>();
  let sub_parts: Vec<String> = parts
    .get(0)
    .ok_or(AppError::MachinesEntryMissingFieldError(
      "dash separating fields from public key".to_string()
    ))?
    .split(" ")
    .map(str::trim)
    .map(|x| x.to_string())
    .collect()
    ;
  debug!("Line sub parts: {:?}", sub_parts);
  Ok(EtcNixMachineRaw {
    url: parse_field_string(
      "url".to_string(),
      sub_parts.get(0),
    )?,
    platforms: parse_field_vec_string(
      ",".to_string(),
      "platforms".to_string(),
      sub_parts.get(1),
    )?,
    public_key_base64: parse_field_string(
      "public_key".to_string(),
      parts.get(1),
    )?,
    private_key_path: parse_field_string(
      "private_key_path".to_string(),
      sub_parts.get(2),
    )?,
    max_jobs: parse_field_u32(
      "max_jobs".to_string(),
      sub_parts.get(3)
    )?,
    speed_factor: parse_field_u32(
      "speed_factor".to_string(),
      sub_parts.get(4),
    )?,
    supported_features: parse_field_vec_string(
      ",".to_string(),
      "supported_features".to_string(),
      sub_parts.get(5),
    )?,
  })
}

pub fn parse_raw(s: String) -> Result<Vec<EtcNixMachineRaw>, AppError> {
  s
    .split("\n")
    .into_iter()
    .filter(|line| !line.is_empty())
    .map(line_to_machine_raw)
    .collect()
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

pub fn ssh_config_value_with_default(
  field: &str,
  original: &str,
  hostname: &str,
) -> Result<String, AppError> {
  ssh_config_value(field, hostname)
    .or_else({|e|
      match e {
        AppError::SshConfigQueryFieldMissingError(_h, _f) => {
          Ok(original.to_string())
        },
        _ => Err(e)
      }
    })
}

// The URL settings can change based on the SSH configuration.  Query `ssh` for
// configuration values for each host.
pub fn url_with_ssh_config(s: &String) -> Result<Url, AppError> {
  let original = Url::parse(s)
    .map_err(AppError::UrlParseError)?;
  let host = original.host_str()
    .ok_or(AppError::UrlHostnameMissingError("Hostname missing.".to_string()))?;
  // Url will barf if given a dotless hostname (like localhost) with the
  // incorrect RelativeUrlWithoutBase error.  Fix this by giving it a protocol,
  // it has some idea as to what we are talking about.
  Url::parse(format!(
    "ssh://{}@{}:{}",
    // TODO: Make sure root is the default Nix uses, and then use it here if so.
    // ssh_config_value("user", original.username(), host)?,
    original.username(),
    ssh_config_value_with_default("hostname", host, host)?,
    ssh_config_value_with_default(
      "port",
      original.port().unwrap_or(22 as u16).to_string().as_str(),
      host,
    )?,
  ).as_str())
    .map_err(AppError::UrlParseError)
}

pub fn parse(x: EtcNixMachineRaw) -> Result<Machine, AppError> {
  // TODO: Split out the file loading and decoding so we can bundle them as
  // joint errors.  We do not want short-circuit behavior here.
  let url = url_with_ssh_config(&x.url)?;
  let url_string = url.to_string();
  let host_str = url
    .host_str()
    .ok_or_else(move || {
      AppError::MachinesEntryUrlHostnameMissingError(url_string)
    })?;
  Ok(Machine {
    url: url.clone(),
    platforms: x.platforms.clone(),
    private_key: private_key_loaded(host_str, &x.private_key_path)
      .inspect(|x| debug!("private_key: {}", x) )?,
    private_key_path: x.private_key_path.clone(),
    // public_key: public_key_decoded(&x.public_key_base64)
    //   .inspect(|x| debug!("public_key: {}", x) )
    //   .map(|x| x.trim_end().to_string())
    //   ?,
    public_key: public_key_loaded(host_str, &x.private_key_path)
      .inspect(|x| debug!("public_key: {}", x) )?,
    max_jobs: x.max_jobs.clone(),
    speed_factor: x.speed_factor.clone(),
    supported_features: x.supported_features.clone(),
  })
}

// This exact code can be put into a lambda and Rust just won't work with it.  I
// give up.  Here's your standalone function, Rust.  The original code:
//  .and_then({|xs|
//    xs
//      .into_iter()
//      .map(machine::parse)
//      .collect::<Result<Vec<Machine>, AppError>>()
//  })
pub fn parse_all(xs: Vec<EtcNixMachineRaw>) -> Vec<Result<Machine, AppError>> {
  xs
    .into_iter()
    .map(parse)
    .collect::<Vec<Result<Machine, AppError>>>()
}

fn public_key_decoded(x: &String) -> Result<String, AppError> {
  BASE64_STANDARD.decode(x)
    .map_err(AppError::PublicKeyDecodeError)
    .and_then({|x|
      std::str::from_utf8(&x)
        .map(|s| s.to_string())
        .map_err(AppError::PublicKeyUtf8Error)
    })
}

fn private_key_loaded(
  hostname: &str,
  path: &String,
) -> Result<String, AppError> {
    fs::read_to_string(private_key_path_infer(hostname, path)?)
      .map_err(AppError::PrivateKeyFileReadError)
}

// The path could be "-" in which case we're supposed to fall back on the SSH
// configuration.
fn private_key_path_infer(
  hostname: &str,
  path: &String
) -> Result<String, AppError> {
  Ok(if path == "-" {
    ssh_config_value("identityfile", hostname)?
  } else {
    path.to_string()
  })
}

fn public_key_path_infer(
  hostname: &str,
  path: &String
) -> Result<String, AppError> {
  Ok(private_key_path_infer(hostname, path)? + ".pub")
}

fn public_key_loaded(
  hostname: &str,
  path: &String,
) -> Result<String, AppError> {
    fs::read_to_string(public_key_path_infer(hostname, path)?)
      .map_err(AppError::PublicKeyFileReadError)
}
