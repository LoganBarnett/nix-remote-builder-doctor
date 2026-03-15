{ config, lib, pkgs, ... }:

let
  cfg = config.services.nix-remote-builder-doctor;

  # The test names as they appear in the code
  testNames = [
    { name = "dns"; displayName = "DNS"; }
    { name = "matching-keys"; displayName = "Matching Keys"; }
    { name = "connection"; displayName = "Connection"; }
    { name = "host-key"; displayName = "Host Key"; }
    { name = "remote-build"; displayName = "Remote Build"; }
    { name = "local-to-remote-build"; displayName = "Local To Remote Build"; }
  ];

  # Generate a check script for a specific builder and test
  # Since nix-remote-builder-doctor doesn't support JSON output yet,
  # we parse the table output to check individual test results
  makeCheckScript = builder: test: pkgs.writeShellScriptBin "nix-remote-builder-doctor-check-${builder.name}-${test.name}" ''
    set -euo pipefail

    # Run the doctor for this specific builder
    OUTPUT=$(${cfg.package}/bin/nix-remote-builder-doctor --include ${builder.hostName} 2>&1)
    EXIT_CODE=$?

    # Debug output
    if [ "''${DEBUG:-0}" = "1" ]; then
      echo "=== Doctor output ==="
      echo "$OUTPUT"
      echo "=== Exit code: $EXIT_CODE ==="
    fi

    # Extract the table from the output
    TABLE=$(echo "$OUTPUT" | ${pkgs.gnugrep}/bin/grep -A 100 "╭─" || true)

    if [ -z "$TABLE" ]; then
      echo "Could not find output table from nix-remote-builder-doctor"
      exit 1
    fi

    # Find the row for our host
    HOST_ROW=$(echo "$TABLE" | ${pkgs.gnugrep}/bin/grep -E "│\s*${builder.hostName}" || true)

    if [ -z "$HOST_ROW" ]; then
      echo "Could not find ${builder.hostName} in output"
      exit 1
    fi

    # The table columns are: Host, DNS, Matching Keys, Connection, Host Key, Remote Build, Local To Remote Build
    # Split the row and get the appropriate column
    case "${test.name}" in
      "dns")
        COLUMN=2
        ;;
      "matching-keys")
        COLUMN=3
        ;;
      "connection")
        COLUMN=4
        ;;
      "host-key")
        COLUMN=5
        ;;
      "remote-build")
        COLUMN=6
        ;;
      "local-to-remote-build")
        COLUMN=7
        ;;
      *)
        echo "Unknown test: ${test.name}"
        exit 1
        ;;
    esac

    # Extract the test result from the appropriate column
    # The row format is: │ hostname │ result │ result │ ...
    TEST_RESULT=$(echo "$HOST_ROW" | ${pkgs.gawk}/bin/awk -F '│' "{print \$$((COLUMN + 1))}" | ${pkgs.coreutils}/bin/tr -d ' ')

    # Check the result
    case "$TEST_RESULT" in
      *"Pass"*)
        echo "✓ ${test.displayName} test passed for ${builder.hostName}"
        exit 0
        ;;
      *"Fail"*)
        echo "✗ ${test.displayName} test failed for ${builder.hostName}"
        exit 1
        ;;
      *"TestRequest"* | *"Inconclusive"*)
        echo "⚠ ${test.displayName} test skipped for ${builder.hostName} (dependency failed)"
        exit 0  # Consider skipped as non-failure for goss
        ;;
      *)
        echo "Unknown test status: '$TEST_RESULT' for ${test.displayName} on ${builder.hostName}"
        exit 1
        ;;
    esac
  '';

  # Generate all check scripts for all builders and tests
  allCheckScripts = lib.flatten (
    map (builder:
      map (test:
        makeCheckScript builder test
      ) testNames
    ) cfg.builders
  );

  # Generate goss configuration for a builder
  makeGossConfig = builder: {
    command = lib.listToAttrs (
      # Overall health check for this builder
      [{
        name = "nix-remote-builder-doctor-${builder.name}";
        value = {
          exec = "${cfg.package}/bin/nix-remote-builder-doctor --include ${builder.hostName}";
          exit-status = 0;
          timeout = cfg.timeout;
        };
      }] ++
      # Individual test checks
      (map (test: {
        name = "nix-remote-builder-doctor-${builder.name}-${test.name}";
        value = {
          exec = "${makeCheckScript builder test}/bin/nix-remote-builder-doctor-check-${builder.name}-${test.name}";
          exit-status = 0;
          timeout = cfg.timeout;
        };
      }) testNames)
    );
  };

in
{
  options.services.nix-remote-builder-doctor = {
    enable = lib.mkEnableOption "nix-remote-builder-doctor service";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.nix-remote-builder-doctor;
      defaultText = lib.literalExpression "pkgs.nix-remote-builder-doctor";
      description = "The nix-remote-builder-doctor package to use.";
    };

    builders = lib.mkOption {
      type = lib.types.listOf (lib.types.submodule {
        options = {
          name = lib.mkOption {
            type = lib.types.str;
            description = "Short name for this builder (used in check names).";
            example = "silicon";
          };
          hostName = lib.mkOption {
            type = lib.types.str;
            description = "Hostname or FQDN of the remote builder.";
            example = "silicon.proton";
          };
        };
      });
      default = [];
      description = "List of remote builders to monitor.";
      example = lib.literalExpression ''
        [
          { name = "silicon"; hostName = "silicon.proton"; }
          { name = "rpi-build"; hostName = "rpi-build.proton"; }
        ]
      '';
    };

    timeout = lib.mkOption {
      type = lib.types.int;
      default = 30000;
      description = "Timeout in milliseconds for each check.";
    };

    integrations.goss = {
      enable = lib.mkEnableOption "goss integration for nix-remote-builder-doctor";

      configFile = lib.mkOption {
        type = lib.types.path;
        readOnly = true;
        description = "Generated goss configuration file path.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    # Ensure the package is available
    environment.systemPackages = [ cfg.package ] ++ lib.optionals cfg.integrations.goss.enable allCheckScripts;

    # Generate goss configuration if enabled
    services.nix-remote-builder-doctor.integrations.goss.configFile = lib.mkIf cfg.integrations.goss.enable (
      pkgs.writeText "nix-remote-builder-doctor-goss.yaml" (
        builtins.toJSON {
          # Merge all builder configs
          command = lib.foldl' (acc: builder:
            acc // (makeGossConfig builder).command
          ) {} cfg.builders;
        }
      )
    );

    # If you have a goss service that can include additional config files, you would add it here
    # For example:
    # services.goss.additionalConfigs = lib.optional cfg.integrations.goss.enable cfg.integrations.goss.configFile;
  };
}