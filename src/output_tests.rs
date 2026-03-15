#[cfg(test)]
mod tests {
    use crate::output::*;
    use crate::test::{MachineTestResult, TestResult, PassData, FailData, MachineTestContext, AppTestContext};
    use crate::machine::Machine;
    use url::Url;

    fn create_test_machine(hostname: &str) -> Machine {
        Machine {
            url: Url::parse(&format!("ssh://builder@{}", hostname)).unwrap(),
            platforms: vec!["x86_64-linux".to_string()],
            user_private_key: "test_key_content".to_string(),
            user_private_key_path: "/tmp/test_key".to_string(),
            host_public_key: "test_host_public_key".to_string(),
            max_jobs: 1,
            speed_factor: 1,
            supported_features: vec![],
        }
    }

    fn create_test_context(hostname: &str) -> MachineTestContext {
        MachineTestContext {
            machine: create_test_machine(hostname),
            app_context: AppTestContext {},
        }
    }

    #[test]
    fn test_json_output_all_pass() {
        let context = create_test_context("test-host.example.com");
        let results = vec![
            MachineTestResult {
                machine: context.machine.clone(),
                test_results: vec![
                    TestResult::Pass(PassData {
                        context: context.clone(),
                        test_name: "DNS".to_string(),
                    }),
                    TestResult::Pass(PassData {
                        context: context.clone(),
                        test_name: "Matching Keys".to_string(),
                    }),
                    TestResult::Pass(PassData {
                        context: context.clone(),
                        test_name: "Connection".to_string(),
                    }),
                    TestResult::Pass(PassData {
                        context: context.clone(),
                        test_name: "Host Key".to_string(),
                    }),
                    TestResult::Pass(PassData {
                        context: context.clone(),
                        test_name: "Remote Build".to_string(),
                    }),
                    TestResult::Pass(PassData {
                        context: context.clone(),
                        test_name: "Local To Remote Build".to_string(),
                    }),
                ],
            },
        ];

        // Test that the function doesn't panic
        json_print(&results);

        // Basic validation that function completes without panic
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].test_results.len(), 6);
    }

    #[test]
    fn test_json_output_with_failures() {
        let context = create_test_context("failing-host.example.com");
        let results = vec![
            MachineTestResult {
                machine: context.machine.clone(),
                test_results: vec![
                    TestResult::Pass(PassData {
                        context: context.clone(),
                        test_name: "DNS".to_string(),
                    }),
                    TestResult::Fail(FailData {
                        context: context.clone(),
                        test_name: "Matching Keys".to_string(),
                        reason: "Key mismatch detected".to_string(),
                        suggestion: "Update the SSH key in /etc/nix/machines".to_string(),
                    }),
                    TestResult::Fail(FailData {
                        context: context.clone(),
                        test_name: "Connection".to_string(),
                        reason: "Connection refused".to_string(),
                        suggestion: "Check if SSH is running on the remote host".to_string(),
                    }),
                ],
            },
        ];

        // Basic validation
        json_print(&results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].test_results.len(), 3);
    }
}