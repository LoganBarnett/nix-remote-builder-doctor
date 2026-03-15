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
  makeCheckScript = builder: test: pkgs.writeShellScriptBin "nix-remote-builder-doctor-check-${builder.name}-${test.name}" ''
    set -euo pipefail

    # Run the doctor for this specific builder with JSON output and specific test
    OUTPUT=$(${cfg.package}/bin/nix-remote-builder-doctor --format json --include ${builder.hostName} --test ${test.name} 2>&1)
    EXIT_CODE=$?

    # Debug output
    if [ "''${DEBUG:-0}" = "1" ]; then
      echo "=== Doctor output ==="
      echo "$OUTPUT"
      echo "=== Exit code: $EXIT_CODE ==="
    fi

    # If the doctor itself failed, report that
    if [ $EXIT_CODE -ne 0 ]; then
      echo "nix-remote-builder-doctor failed to run: $OUTPUT"
      exit 1
    fi

    # Parse JSON output to check if this specific test passed
    TEST_STATUS=$(echo "$OUTPUT" | ${pkgs.jq}/bin/jq -r --arg test "${test.displayName}" '
      .builders[] |
      select(.hostname == "${builder.hostName}") |
      .checks[] |
      select(.name == $test) |
      .status
    ')

    # Handle missing test result
    if [ -z "$TEST_STATUS" ]; then
      echo "Test '${test.displayName}' not found in output for ${builder.hostName}"
      exit 1
    fi

    # Exit based on test status
    case "$TEST_STATUS" in
      "pass")
        echo "✓ ${test.displayName} test passed for ${builder.hostName}"
        exit 0
        ;;
      "skip")
        # Get the skip reason if available
        REASON=$(echo "$OUTPUT" | ${pkgs.jq}/bin/jq -r --arg test "${test.displayName}" '
          .builders[] |
          select(.hostname == "${builder.hostName}") |
          .checks[] |
          select(.name == $test) |
          .message // "Dependency failed"
        ')
        echo "⚠ ${test.displayName} test skipped for ${builder.hostName}: $REASON"
        exit 0  # Consider skipped as non-failure for goss
        ;;
      "fail")
        # Get the failure details
        FAILURE_INFO=$(echo "$OUTPUT" | ${pkgs.jq}/bin/jq -r --arg test "${test.displayName}" '
          .builders[] |
          select(.hostname == "${builder.hostName}") |
          .checks[] |
          select(.name == $test) |
          "Reason: " + (.reason // "Unknown") + "\nSuggestion: " + (.suggestion // "Check configuration")
        ')
        echo "✗ ${test.displayName} test failed for ${builder.hostName}"
        echo "$FAILURE_INFO"
        exit 1
        ;;
      *)
        echo "Unknown test status: $TEST_STATUS"
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
          exec = "${cfg.package}/bin/nix-remote-builder-doctor --format json --include ${builder.hostName}";
          exit-status = 0;
          timeout = cfg.timeout;
          stdout = [
            "/\"overall_status\":\\s*\"healthy\"/"
          ];
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