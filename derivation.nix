{ fetchFromGitHub, pkgs ? import <nixpkgs> {}, lib, rustPlatform, stdenv }:

rustPlatform.buildRustPackage {
  pname = "nix-remote-builder-doctor";
  version = "0.0.0";
  src = fetchFromGitHub {
    owner = "LoganBarnett";
    repo = "nix-remote-builder-doctor";
    hash = "";
  };
  cargoHash = "";
  meta = {
    description = "A doctor app for diagnosing issues with remote building on Nix.";
    homepage = "https://github.com/LoganBarnett/nix-remote-builder-doctor";
    license = lib.licenses.mit;
    maintainers = [];
  };
}
