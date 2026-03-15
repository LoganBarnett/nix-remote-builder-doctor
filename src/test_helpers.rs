#[cfg(test)]
pub mod test_helpers {
    use crate::machine::Machine;
    use crate::test::{TestResult, PassData, FailData, MachineTestContext, AppTestContext};
    use url::Url;

    pub fn create_test_machine(hostname: &str) -> Machine {
        Machine {
            url: Url::parse(&format!("ssh://builder@{}", hostname)).unwrap(),
            systems: vec!["x86_64-linux".to_string()],
            ssh_key: "/tmp/test_key".to_string(),
            max_jobs: 1,
            speed_factor: 1,
            supported_features: vec![],
            mandatory_features: vec![],
            public_host_key: None,
        }
    }

    pub fn create_test_context(hostname: &str) -> MachineTestContext {
        MachineTestContext {
            machine: create_test_machine(hostname),
            app_context: AppTestContext {},
        }
    }

    pub fn assert_test_passes(result: &TestResult, test_name: &str) {
        match result {
            TestResult::Pass(data) => {
                assert_eq!(data.test_name, test_name);
            },
            _ => panic!("Expected {} test to pass, but it didn't", test_name),
        }
    }

    pub fn assert_test_fails(result: &TestResult, test_name: &str) {
        match result {
            TestResult::Fail(data) => {
                assert_eq!(data.test_name, test_name);
            },
            _ => panic!("Expected {} test to fail, but it didn't", test_name),
        }
    }
}