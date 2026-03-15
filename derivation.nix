{ fetchFromGitHub, pkgs ? import <nixpkgs> {}, lib, rustPlatform, stdenv }:

rustPlatform.buildRustPackage {
  pname = "nix-remote-builder-doctor";
  version = "0.0.0";
  src = ./.;
  cargoHash = "sha256-YYhT35OAKZxBgqZto33/Wq4pMBduTQjD0uMP/qOcYhI=";
  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ pkgs.openssl pkgs.libssh2 ];

  # Disable integration tests in Nix build to avoid sandbox violations
  checkPhase = ''
    cargo test --lib --bins
  '';
  meta = {
    description = "A doctor app for diagnosing issues with remote building on Nix.";
    homepage = "https://github.com/LoganBarnett/nix-remote-builder-doctor";
    license = lib.licenses.mit;
    maintainers = [];
  };
}
