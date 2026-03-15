use std::fs::{self, File, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub machines_file: PathBuf,
    pub ssh_key: PathBuf,
    pub ssh_port: u16,
    sshd_process: Option<Child>,
}

pub struct TestBuilder {
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub systems: Vec<String>,
    pub ssh_key: String,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ssh_port = portpicker::pick_unused_port().expect("No free port available");

        Self {
            temp_dir,
            machines_file: PathBuf::new(),
            ssh_key: PathBuf::new(),
            ssh_port,
            sshd_process: None,
        }
    }

    pub fn with_builder(mut self, builder: TestBuilder) -> Self {
        // Create directory structure
        let etc_nix = self.temp_dir.path().join("etc/nix");
        let ssh_dir = self.temp_dir.path().join("ssh");
        let bin_dir = self.temp_dir.path().join("bin");

        fs::create_dir_all(&etc_nix).expect("Failed to create etc/nix");
        fs::create_dir_all(&ssh_dir).expect("Failed to create ssh dir");
        fs::create_dir_all(&bin_dir).expect("Failed to create bin dir");

        // Generate SSH key pair
        let key_path = ssh_dir.join("id_ed25519");
        let pub_key_path = ssh_dir.join("id_ed25519.pub");

        Command::new("ssh-keygen")
            .args(&[
                "-t", "ed25519",
                "-f", key_path.to_str().unwrap(),
                "-N", "", // No passphrase
                "-C", "test-builder",
            ])
            .output()
            .expect("Failed to generate SSH key");

        // Set proper permissions on private key
        fs::set_permissions(&key_path, Permissions::from_mode(0o600))
            .expect("Failed to set key permissions");

        // Create authorized_keys
        let pub_key = fs::read_to_string(&pub_key_path)
            .expect("Failed to read public key");
        let authorized_keys = ssh_dir.join("authorized_keys");
        fs::write(&authorized_keys, &pub_key)
            .expect("Failed to write authorized_keys");

        // Create machines file
        self.machines_file = etc_nix.join("machines");
        let machines_entry = format!(
            "ssh://builder@{}:{} {} {} 1 1 - -",
            builder.hostname,
            self.ssh_port,
            builder.systems.join(","),
            key_path.display()
        );
        fs::write(&self.machines_file, machines_entry)
            .expect("Failed to write machines file");

        self.ssh_key = key_path;
        self
    }

    pub fn with_mock_nix(self) -> Self {
        let bin_dir = self.temp_dir.path().join("bin");

        // Create mock nix executable
        let nix_path = bin_dir.join("nix");
        let nix_content = r#"#!/usr/bin/env bash
case "$1" in
  "build")
    echo "building on remote host..."
    echo "/nix/store/fake-result"
    exit 0
    ;;
  "copy")
    echo "copying to remote host..."
    exit 0
    ;;
  *)
    echo "mock nix: $@" >&2
    exit 0
    ;;
esac
"#;
        fs::write(&nix_path, nix_content).expect("Failed to write mock nix");
        fs::set_permissions(&nix_path, Permissions::from_mode(0o755))
            .expect("Failed to set nix permissions");

        // Create mock nix-daemon
        let nix_daemon_path = bin_dir.join("nix-daemon");
        let nix_daemon_content = r#"#!/usr/bin/env bash
echo "nix-daemon (mock) running"
exit 0
"#;
        fs::write(&nix_daemon_path, nix_daemon_content)
            .expect("Failed to write mock nix-daemon");
        fs::set_permissions(&nix_daemon_path, Permissions::from_mode(0o755))
            .expect("Failed to set nix-daemon permissions");

        self
    }

    pub fn start(mut self) -> Self {
        let ssh_dir = self.temp_dir.path().join("ssh");
        let sshd_config = ssh_dir.join("sshd_config");

        // Generate host key
        let host_key_path = ssh_dir.join("ssh_host_ed25519_key");
        Command::new("ssh-keygen")
            .args(&[
                "-t", "ed25519",
                "-f", host_key_path.to_str().unwrap(),
                "-N", "",
            ])
            .output()
            .expect("Failed to generate host key");

        // Create sshd_config
        let config_content = format!(r#"
Port {}
ListenAddress 127.0.0.1
HostKey {}
PidFile {}
AuthorizedKeysFile {}
PasswordAuthentication no
PubkeyAuthentication yes
ChallengeResponseAuthentication no
UsePAM no
StrictModes no
"#,
            self.ssh_port,
            host_key_path.display(),
            ssh_dir.join("sshd.pid").display(),
            ssh_dir.join("authorized_keys").display(),
        );

        fs::write(&sshd_config, config_content)
            .expect("Failed to write sshd_config");

        // Start sshd
        let mut child = Command::new("/usr/sbin/sshd")
            .args(&[
                "-D", // Don't detach
                "-f", sshd_config.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start sshd");

        // Give sshd time to start
        thread::sleep(Duration::from_millis(500));

        self.sshd_process = Some(child);
        self
    }

    pub fn run_doctor(&self, args: &[&str]) -> String {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_nix-remote-builder-doctor"));
        cmd.env("PATH", self.temp_dir.path().join("bin"))
            .arg("--machines-file")
            .arg(&self.machines_file)
            .args(args);

        let output = cmd.output().expect("Failed to run doctor");

        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 in output")
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.sshd_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_creation() {
        let env = TestEnvironment::new();
        assert!(env.temp_dir.path().exists());
    }
}