{
  description = "Diagnose Nix remote builder issues.";
  inputs = {
    # LLM: Do NOT change this URL unless explicitly directed.  This is the
    # correct format for nixpkgs stable (25.11 is correct, not nixos-25.11).
    nixpkgs.url = "github:NixOS/nixpkgs/25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    changelog-roller.url = "github:LoganBarnett/changelog-roller";
    foundation.url = "github:LoganBarnett/rust-template";
    foundation.inputs.nixpkgs.follows = "nixpkgs";
    org-fmt.url = "github:LoganBarnett/org-fmt";
    org-fmt.inputs.nixpkgs.follows = "nixpkgs";
    org-fmt.inputs.rust-overlay.follows = "rust-overlay";
    org-fmt.inputs.crane.follows = "crane";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
    changelog-roller,
    foundation,
    org-fmt,
  } @ inputs: let
    forAllSystems =
      nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    perSystem = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      craneLib =
        (crane.mkLib pkgs).overrideToolchain
        (p: p.rust-bin.stable.latest.default);
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
          "rustfmt"
        ];
      };
      crates = {
        # CRATE:cli:begin
        cli = {
          name = "nix-remote-builder-doctor";
          binary = "nix-remote-builder-doctor";
          description = "Diagnose Nix remote builder issues";
        };
        # CRATE:cli:end
        # CRATE_ENTRIES
      };
      # libssh2 + openssl are link-time requirements of the `ssh2`
      # crate; pkg-config + clang resolve them at build time.  Carried
      # forward from the pre-template derivation.nix.
      sshBuildInputs = [
        pkgs.openssl
        pkgs.libssh2
      ];
      sshNativeBuildInputs = [
        pkgs.clang
        pkgs.pkg-config
      ];
      commonArgs = {
        src = craneLib.cleanCargoSource self;
        buildInputs = sshBuildInputs;
        nativeBuildInputs = sshNativeBuildInputs;
        # Run only bin-target tests; integration tests under tests/
        # spin up an in-sandbox sshd and need network access the Nix
        # sandbox does not grant.  `--lib` is omitted because the cli
        # crate is bin-only — `cargo test --lib -p <bin-only>` errors
        # with "no library targets found".  Lib-crate unit tests run
        # via `cargo test --workspace` in CI and the devshell.
        cargoTestExtraArgs = "--bins";
      };
      rustPackages = foundation.lib.mkRustPackages {
        inherit self pkgs craneLib crates commonArgs;
      };
    in {
      packages =
        rustPackages.packages
        // {
          default = rustPackages.packages.cli;
        };
      inherit (rustPackages) apps;
      devShell = pkgs.mkShell {
        buildInputs =
          [
            rust
            pkgs.cargo-sweep
            pkgs.jq
            pkgs.treefmt
            pkgs.alejandra
            pkgs.prettier
            pkgs.just
            changelog-roller.packages.${system}.default
            org-fmt.packages.${system}.default
            (pkgs.cargo-semver-checks.overrideAttrs (_: {doCheck = false;}))
          ]
          ++ sshBuildInputs
          ++ sshNativeBuildInputs;
        shellHook = ''
          ${foundation.lib.cargoHuskyHookSnippet pkgs}
          echo "nix-remote-builder-doctor development environment"
          echo ""
          echo "Available Cargo packages (use 'cargo build -p <name>'):"
          cargo metadata --no-deps --format-version 1 2>/dev/null | \
            jq --raw-output '.packages[].name' | \
            sort | \
            sed 's/^/  • /' || echo "  Run 'cargo init' to get started"
        '';
      };
    });
  in {
    devShells =
      nixpkgs.lib.mapAttrs (_: p: {default = p.devShell;}) perSystem;
    packages = nixpkgs.lib.mapAttrs (_: p: p.packages) perSystem;
    apps = nixpkgs.lib.mapAttrs (_: p: p.apps) perSystem;

    # Project-specific NixOS and Darwin modules.  The template's
    # `mkNixosService` helper targets server-style daemons; this project
    # ships a CLI that the modules wire into goss-driven health checks,
    # so we keep the existing module bodies and just point the flake at
    # them under the template-conventional `nix/modules/` location.
    nixosModules.default = ./nix/modules/nixos.nix;
    darwinModules.default = ./nix/modules/darwin.nix;
  };
}
