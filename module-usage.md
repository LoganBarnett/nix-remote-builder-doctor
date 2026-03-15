# NixOS Module Usage

This flake provides a NixOS module for integrating `nix-remote-builder-doctor` with your system, including optional goss integration for monitoring.

## Basic Usage

Add the flake to your inputs:

```nix
{
  inputs = {
    nix-remote-builder-doctor.url = "github:your-username/nix-remote-builder-doctor";
  };
}
```

Then in your NixOS configuration:

```nix
{ inputs, ... }:
{
  imports = [ inputs.nix-remote-builder-doctor.nixosModules.default ];

  services.nix-remote-builder-doctor = {
    enable = true;
    builders = [
      { name = "silicon"; hostName = "silicon.proton"; }
      { name = "rpi-build"; hostName = "rpi-build.proton"; }
    ];
  };
}
```

## Goss Integration

The module can generate goss check scripts for each builder and test:

```nix
{
  services.nix-remote-builder-doctor = {
    enable = true;
    builders = [
      { name = "silicon"; hostName = "silicon.proton"; }
      { name = "rpi-build"; hostName = "rpi-build.proton"; }
    ];
    integrations.goss.enable = true;
  };
}
```

This will create the following goss checks:

### Overall health checks per builder:
- `nix-remote-builder-doctor-silicon`
- `nix-remote-builder-doctor-rpi-build`

### Individual test checks per builder:
- `nix-remote-builder-doctor-silicon-dns`
- `nix-remote-builder-doctor-silicon-matching-keys`
- `nix-remote-builder-doctor-silicon-connection`
- `nix-remote-builder-doctor-silicon-host-key`
- `nix-remote-builder-doctor-silicon-remote-build`
- `nix-remote-builder-doctor-silicon-local-to-remote-build`

And similarly for each other builder.

## Configuration Options

### `services.nix-remote-builder-doctor.enable`
Enable the nix-remote-builder-doctor service.

### `services.nix-remote-builder-doctor.package`
The package to use (defaults to `pkgs.nix-remote-builder-doctor`).

### `services.nix-remote-builder-doctor.builders`
List of remote builders to monitor. Each builder should have:
- `name`: Short identifier used in check names
- `hostName`: FQDN or hostname of the remote builder

### `services.nix-remote-builder-doctor.timeout`
Timeout in milliseconds for each check (default: 30000).

### `services.nix-remote-builder-doctor.integrations.goss.enable`
Enable goss integration to generate check scripts.

### `services.nix-remote-builder-doctor.integrations.goss.configFile`
Read-only option that contains the path to the generated goss configuration.

## Implementation Notes

Since `nix-remote-builder-doctor` doesn't currently support JSON output, the individual test check scripts parse the table output. The scripts look for specific test results in the output table and return appropriate exit codes.

The check scripts support a `DEBUG=1` environment variable to show the raw doctor output for troubleshooting.

## Example Goss Output

When integrated with goss-exporter, you'll get Prometheus metrics like:

```
goss_check{test="nix-remote-builder-doctor-silicon"} 1
goss_check{test="nix-remote-builder-doctor-silicon-dns"} 1
goss_check{test="nix-remote-builder-doctor-silicon-matching-keys"} 1
goss_check{test="nix-remote-builder-doctor-silicon-connection"} 1
goss_check{test="nix-remote-builder-doctor-silicon-host-key"} 1
goss_check{test="nix-remote-builder-doctor-silicon-remote-build"} 1
goss_check{test="nix-remote-builder-doctor-silicon-local-to-remote-build"} 1
goss_check{test="nix-remote-builder-doctor-rpi-build"} 0
goss_check{test="nix-remote-builder-doctor-rpi-build-dns"} 1
goss_check{test="nix-remote-builder-doctor-rpi-build-matching-keys"} 1
goss_check{test="nix-remote-builder-doctor-rpi-build-connection"} 0
```

Where:
- `1` = test passed
- `0` = test failed or couldn't run

Note: Skipped tests (due to dependency failures) are treated as passing (1) to avoid cascading alerts.