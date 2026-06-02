# NixOS entry point for the nix-remote-builder-doctor module.  The
# option schema and goss check-script generation live in
# `./common.nix`; this file is the seam where NixOS-only declarations
# (e.g. systemd drop-ins) would go if they ever became necessary.
# Anything added below merges with the common module via NixOS's
# module-merge system — no restructuring is required to introduce a
# platform-specific bit.
{...}: {
  imports = [./common.nix];
}
