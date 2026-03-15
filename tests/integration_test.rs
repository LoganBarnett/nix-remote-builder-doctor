mod common;

use common::{TestBuilder, TestEnvironment};
use serde_json::Value;

#[test]
#[ignore] // Run with: cargo test --test integration_test -- --ignored
fn test_dns_resolution_passes() {
    let env = TestEnvironment::new()
        .with_builder(TestBuilder {
            name: "test-builder".to_string(),
            hostname: "localhost".to_string(),
            port: 0, // Will be set by TestEnvironment
            systems: vec!["x86_64-linux".to_string()],
            ssh_key: String::new(),
        })
        .with_mock_nix()
        .start();

    let output = env.run_doctor(&["--format", "json"]);
    let json: Value = serde_json::from_str(&output)
        .expect("Failed to parse JSON output");

    // Check overall health
    assert_eq!(
        json["summary"]["overall"].as_str().unwrap(),
        "healthy",
        "Expected overall status to be healthy"
    );

    // Check DNS test specifically
    let builder = &json["builders"][0];
    let dns_check = builder["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "DNS")
        .expect("DNS check not found");

    assert_eq!(
        dns_check["status"].as_str().unwrap(),
        "pass",
        "DNS check should pass for localhost"
    );
}

#[test]
#[ignore]
fn test_specific_test_filtering() {
    let env = TestEnvironment::new()
        .with_builder(TestBuilder {
            name: "test-builder".to_string(),
            hostname: "localhost".to_string(),
            port: 0,
            systems: vec!["x86_64-linux".to_string()],
            ssh_key: String::new(),
        })
        .with_mock_nix()
        .start();

    // Run only DNS test
    let output = env.run_doctor(&["--format", "json", "--test", "dns"]);
    let json: Value = serde_json::from_str(&output)
        .expect("Failed to parse JSON output");

    let builder = &json["builders"][0];
    let checks = builder["checks"].as_array().unwrap();

    // Should only have one check
    assert_eq!(checks.len(), 1, "Should only run DNS test");
    assert_eq!(checks[0]["name"], "DNS");
}

#[test]
#[ignore]
fn test_table_output_format() {
    let env = TestEnvironment::new()
        .with_builder(TestBuilder {
            name: "test-builder".to_string(),
            hostname: "localhost".to_string(),
            port: 0,
            systems: vec!["x86_64-linux".to_string()],
            ssh_key: String::new(),
        })
        .with_mock_nix()
        .start();

    let output = env.run_doctor(&["--format", "table"]);

    // Check for table borders
    assert!(output.contains("╭"), "Should contain table top border");
    assert!(output.contains("│"), "Should contain table columns");
    assert!(output.contains("Host"), "Should contain Host header");
    assert!(output.contains("DNS"), "Should contain DNS header");
}

#[test]
#[ignore]
fn test_nonexistent_host() {
    let env = TestEnvironment::new()
        .with_builder(TestBuilder {
            name: "test-builder".to_string(),
            hostname: "nonexistent.invalid".to_string(),
            port: 0,
            systems: vec!["x86_64-linux".to_string()],
            ssh_key: String::new(),
        })
        .with_mock_nix()
        .start();

    let output = env.run_doctor(&["--format", "json"]);
    let json: Value = serde_json::from_str(&output)
        .expect("Failed to parse JSON output");

    let builder = &json["builders"][0];
    let dns_check = builder["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "DNS")
        .expect("DNS check not found");

    assert_eq!(
        dns_check["status"].as_str().unwrap(),
        "fail",
        "DNS check should fail for nonexistent host"
    );
}

// Helper function to run integration tests locally
// Run with: cargo test --test integration_test -- --ignored --nocapture
#[cfg(test)]
fn main() {
    println!("Run integration tests with: cargo test --test integration_test -- --ignored");
}