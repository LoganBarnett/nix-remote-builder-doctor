{ fetchFromGitHub, pkgs ? import <nixpkgs> {}, lib, rustPlatform, stdenv }:

rustPlatform.buildRustPackage {
  pname = "nix-remote-builder-doctor";
  version = "0.0.0";
  src = ./.;
  cargoHash = "sha256-AgeWwgLgzDSfHzyzE52293xv+Mr5pso5xlvjF9/UWjo=";
  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ pkgs.openssl pkgs.libssh2 ];
  meta = {
    description = "A doctor app for diagnosing issues with remote building on Nix.";
    homepage = "https://github.com/LoganBarnett/nix-remote-builder-doctor";
    license = lib.licenses.mit;
    maintainers = [];
  };
}
