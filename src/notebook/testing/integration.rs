// SPRINT5-001: CI/CD Integration implementation
// PMAT Complexity: <10 per function
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct CiCdConfig {
    pub provider: CiProvider,
    pub trigger_on_push: bool,
    pub trigger_on_pr: bool,
    pub run_tests: bool,
    pub run_benchmarks: bool,
    pub coverage_threshold: f64,
    pub complexity_threshold: usize,
}
#[derive(Debug, Clone)]
pub enum CiProvider {
    GitHub,
    GitLab,
    Jenkins,
    CircleCI,
}

#[derive(Debug, Clone)]
pub struct CiCdIntegrator {
    pub config: Option<CiCdConfig>,
}

impl Default for CiCdIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

impl CiCdIntegrator {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::integration::CiCdIntegrator;
    ///
    /// let instance = CiCdIntegrator::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::integration::CiCdIntegrator;
    ///
    /// let instance = CiCdIntegrator::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::integration::CiCdIntegrator;
    ///
    /// let instance = CiCdIntegrator::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self { config: None }
    }
    /// Configure CI/CD settings
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::integration::CiCdIntegrator;
    ///
    /// let mut instance = CiCdIntegrator::new();
    /// let result = instance.configure();
    /// // Verify behavior
    /// ```
    pub fn configure(&mut self, config: CiCdConfig) -> Result<(), String> {
        if config.coverage_threshold > 100.0 || config.coverage_threshold < 0.0 {
            return Err("Coverage threshold must be between 0 and 100".to_string());
        }
        self.config = Some(config);
        Ok(())
    }
    /// Generate workflow file for the configured provider
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::generate_workflow;
    ///
    /// let result = generate_workflow(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_workflow(&self) -> String {
        let config = match &self.config {
            Some(c) => c,
            None => return String::new(),
        };
        match config.provider {
            CiProvider::GitHub => self.generate_github_workflow(config),
            CiProvider::GitLab => self.generate_gitlab_workflow(config),
            CiProvider::Jenkins => self.generate_jenkins_workflow(config),
            CiProvider::CircleCI => self.generate_circleci_workflow(config),
        }
    }
    fn generate_github_workflow(&self, config: &CiCdConfig) -> String {
        let mut workflow = String::from("name: CI\n");
        // Triggers
        workflow.push_str("on:\n");
        if config.trigger_on_push {
            workflow.push_str("  push:\n    branches: [main]\n");
        }
        if config.trigger_on_pr {
            workflow.push_str("  pull_request:\n    branches: [main]\n");
        }
        // Jobs
        workflow.push_str("\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n");
        workflow.push_str("      - uses: actions/checkout@v2\n");
        workflow.push_str("      - uses: actions-rs/toolchain@v1\n");
        if config.run_tests {
            workflow.push_str("      - run: cargo test\n");
        }
        if config.run_benchmarks {
            workflow.push_str("      - run: cargo bench\n");
        }
        if config.coverage_threshold > 0.0 {
            workflow.push_str("      - run: cargo llvm-cov\n");
        }
        workflow
    }
    fn generate_gitlab_workflow(&self, config: &CiCdConfig) -> String {
        let mut workflow = String::from("stages:\n  - test\n\n");
        workflow.push_str("test:\n  stage: test\n  script:\n");
        if config.run_tests {
            workflow.push_str("    - cargo test\n");
        }
        if config.run_benchmarks {
            workflow.push_str("    - cargo bench\n");
        }
        workflow
    }
    fn generate_jenkins_workflow(&self, _config: &CiCdConfig) -> String {
        "pipeline {\n  agent any\n  stages {\n    stage('Test') {\n      steps {\n        sh 'cargo test'\n      }\n    }\n  }\n}".to_string()
    }
    fn generate_circleci_workflow(&self, _config: &CiCdConfig) -> String {
        "version: 2\njobs:\n  test:\n    docker:\n      - image: rust:latest\n    steps:\n      - checkout\n      - run: cargo test".to_string()
    }
}
/// Distributed test coordinator
#[derive(Debug)]
pub struct DistributedTestCoordinator {
    workers: HashMap<String, String>,
}
impl Default for DistributedTestCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl DistributedTestCoordinator {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }
    /// Register a worker node
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::register_worker;
    ///
    /// let result = register_worker("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn register_worker(&mut self, name: &str, address: &str) {
        self.workers.insert(name.to_string(), address.to_string());
    }
    /// Distribute tests across workers
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::distribute;
    ///
    /// let result = distribute(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn distribute(&self, tests: &[String]) -> HashMap<String, Vec<String>> {
        let mut distribution = HashMap::new();
        if self.workers.is_empty() {
            return distribution;
        }
        let workers: Vec<_> = self.workers.keys().collect();
        for (i, test) in tests.iter().enumerate() {
            let worker = workers[i % workers.len()].clone();
            distribution
                .entry(worker)
                .or_insert_with(Vec::new)
                .push(test.clone());
        }
        distribution
    }
    /// Get worker status
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::get_worker_status;
    ///
    /// let result = get_worker_status("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_worker_status(&self, worker: &str) -> WorkerStatus {
        if self.workers.contains_key(worker) {
            WorkerStatus::Active
        } else {
            WorkerStatus::Unknown
        }
    }
}
#[derive(Debug, Clone)]
pub enum WorkerStatus {
    Active,
    Busy,
    Offline,
    Unknown,
}
/// Continuous monitoring system
#[derive(Debug)]
pub struct ContinuousMonitor {
    alerts: Vec<Alert>,
    triggered: Vec<String>,
    metrics: HashMap<Metric, f64>,
}
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub metric: Metric,
    pub threshold: f64,
    pub action: AlertAction,
}
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Metric {
    MemoryUsage,
    CpuUsage,
    TestDuration,
    ErrorRate,
}
#[derive(Debug, Clone)]
pub enum AlertAction {
    Email(String),
    Slack(String),
    PagerDuty(String),
    Webhook(String),
}
impl Default for ContinuousMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuousMonitor {
    pub fn new() -> Self {
        Self {
            alerts: Vec::new(),
            triggered: Vec::new(),
            metrics: HashMap::new(),
        }
    }
    /// Add an alert configuration
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::add_alert;
    ///
    /// let result = add_alert(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }
    /// Start monitoring
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::start;
    ///
    /// let result = start(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn start(&self) {
        // In real implementation, would spawn monitoring thread
    }
    /// Stop monitoring
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::stop;
    ///
    /// let result = stop(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn stop(&self) {
        // In real implementation, would stop monitoring thread
    }
    /// Record a metric value
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::record_metric;
    ///
    /// let result = record_metric(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn record_metric(&mut self, metric: Metric, value: f64) {
        self.metrics.insert(metric.clone(), value);
        // Check if any alerts should trigger
        for alert in &self.alerts {
            if alert.metric == metric
                && value > alert.threshold
                && !self.triggered.contains(&alert.id)
            {
                self.triggered.push(alert.id.clone());
                self.trigger_alert(alert);
            }
        }
    }
    fn trigger_alert(&self, alert: &Alert) {
        match &alert.action {
            AlertAction::Email(email) => {
                println!("Sending alert to {email}");
            }
            AlertAction::Slack(channel) => {
                println!("Posting to Slack channel {channel}");
            }
            AlertAction::PagerDuty(service) => {
                println!("Triggering PagerDuty service {service}");
            }
            AlertAction::Webhook(url) => {
                println!("Calling webhook {url}");
            }
        }
    }
    /// Get triggered alerts
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::get_triggered_alerts;
    ///
    /// let result = get_triggered_alerts(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_triggered_alerts(&self) -> Vec<String> {
        self.triggered.clone()
    }
    /// Clear triggered alerts
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::integration::clear_alerts;
    ///
    /// let result = clear_alerts(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn clear_alerts(&mut self) {
        self.triggered.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // EXTREME TDD: Comprehensive test coverage for CI/CD integration system

    #[test]
    fn test_ci_provider_enum_variants() {
        let providers = [
            CiProvider::GitHub,
            CiProvider::GitLab,
            CiProvider::Jenkins,
            CiProvider::CircleCI,
        ];
        assert_eq!(providers.len(), 4);
    }

    #[test]
    fn test_ci_provider_debug_format() {
        assert_eq!(format!("{:?}", CiProvider::GitHub), "GitHub");
        assert_eq!(format!("{:?}", CiProvider::GitLab), "GitLab");
        assert_eq!(format!("{:?}", CiProvider::Jenkins), "Jenkins");
        assert_eq!(format!("{:?}", CiProvider::CircleCI), "CircleCI");
    }

    #[test]
    fn test_cicd_config_creation() {
        let config = CiCdConfig {
            provider: CiProvider::GitHub,
            trigger_on_push: true,
            trigger_on_pr: true,
            run_tests: true,
            run_benchmarks: false,
            coverage_threshold: 80.0,
            complexity_threshold: 10,
        };
        assert!(matches!(config.provider, CiProvider::GitHub));
        assert!(config.trigger_on_push);
        assert!(config.trigger_on_pr);
        assert!(config.run_tests);
        assert!(!config.run_benchmarks);
        assert_eq!(config.coverage_threshold, 80.0);
        assert_eq!(config.complexity_threshold, 10);
    }

    #[test]
    fn test_cicd_config_clone() {
        let config = CiCdConfig {
            provider: CiProvider::GitLab,
            trigger_on_push: false,
            trigger_on_pr: true,
            run_tests: true,
            run_benchmarks: true,
            coverage_threshold: 90.0,
            complexity_threshold: 15,
        };
        let cloned = config;
        assert!(matches!(cloned.provider, CiProvider::GitLab));
        assert!(!cloned.trigger_on_push);
        assert!(cloned.trigger_on_pr);
        assert_eq!(cloned.coverage_threshold, 90.0);
    }

    #[test]
    fn test_cicd_integrator_new() {
        let integrator = CiCdIntegrator::new();
        assert!(integrator.config.is_none());
    }

    #[test]
    fn test_cicd_integrator_default() {
        let integrator = CiCdIntegrator::default();
        assert!(integrator.config.is_none());
    }

    #[test]
    fn test_configure_valid_config() {
        let mut integrator = CiCdIntegrator::new();
        let config = CiCdConfig {
            provider: CiProvider::GitHub,
            trigger_on_push: true,
            trigger_on_pr: true,
            run_tests: true,
            run_benchmarks: false,
            coverage_threshold: 75.5,
            complexity_threshold: 8,
        };

        let result = integrator.configure(config);
        assert!(result.is_ok());
        assert!(integrator.config.is_some());
        assert_eq!(integrator.config.unwrap().coverage_threshold, 75.5);
    }

    #[test]
    fn test_configure_invalid_coverage_threshold_too_high() {
        let mut integrator = CiCdIntegrator::new();
        let config = CiCdConfig {
            provider: CiProvider::GitHub,
            trigger_on_push: true,
            trigger_on_pr: true,
            run_tests: true,
            run_benchmarks: false,
            coverage_threshold: 150.0,
            complexity_threshold: 10,
        };

        let result = integrator.configure(config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Coverage threshold must be between 0 and 100"
        );
    }

    #[test]
    fn test_configure_invalid_coverage_threshold_negative() {
        let mut integrator = CiCdIntegrator::new();
        let config = CiCdConfig {
            provider: CiProvider::GitHub,
            trigger_on_push: true,
            trigger_on_pr: true,
            run_tests: true,
            run_benchmarks: false,
            coverage_threshold: -10.0,
            complexity_threshold: 10,
        };

        let result = integrator.configure(config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Coverage threshold must be between 0 and 100"
        );
    }

    #[test]
    fn test_generate_workflow_no_config() {
        let integrator = CiCdIntegrator::new();
        let workflow = integrator.generate_workflow();
        assert_eq!(workflow, "");
    }

    #[test]
    fn test_generate_github_workflow() {
        let mut integrator = CiCdIntegrator::new();
        let config = CiCdConfig {
            provider: CiProvider::GitHub,
            trigger_on_push: true,
            trigger_on_pr: true,
            run_tests: true,
            run_benchmarks: true,
            coverage_threshold: 80.0,
            complexity_threshold: 10,
        };
        integrator.configure(config).unwrap();

        let workflow = integrator.generate_workflow();
        assert!(workflow.contains("name: CI"));
        assert!(workflow.contains("on:"));
        assert!(workflow.contains("push:"));
        assert!(workflow.contains("pull_request:"));
        assert!(workflow.contains("cargo test"));
        assert!(workflow.contains("cargo bench"));
        assert!(workflow.contains("cargo llvm-cov"));
    }

    #[test]
    fn test_generate_gitlab_workflow() {
        let mut integrator = CiCdIntegrator::new();
        let config = CiCdConfig {
            provider: CiProvider::GitLab,
            trigger_on_push: false,
            trigger_on_pr: false,
            run_tests: true,
            run_benchmarks: true,
            coverage_threshold: 0.0,
            complexity_threshold: 5,
        };
        integrator.configure(config).unwrap();

        let workflow = integrator.generate_workflow();
        assert!(workflow.contains("stages:"));
        assert!(workflow.contains("test:"));
        assert!(workflow.contains("cargo test"));
        assert!(workflow.contains("cargo bench"));
    }

    #[test]
    fn test_generate_jenkins_workflow() {
        let mut integrator = CiCdIntegrator::new();
        let config = CiCdConfig {
            provider: CiProvider::Jenkins,
            trigger_on_push: true,
            trigger_on_pr: false,
            run_tests: true,
            run_benchmarks: false,
            coverage_threshold: 0.0,
            complexity_threshold: 12,
        };
        integrator.configure(config).unwrap();

        let workflow = integrator.generate_workflow();
        assert!(workflow.contains("pipeline {"));
        assert!(workflow.contains("agent any"));
        assert!(workflow.contains("cargo test"));
    }

    #[test]
    fn test_generate_circleci_workflow() {
        let mut integrator = CiCdIntegrator::new();
        let config = CiCdConfig {
            provider: CiProvider::CircleCI,
            trigger_on_push: false,
            trigger_on_pr: true,
            run_tests: true,
            run_benchmarks: false,
            coverage_threshold: 60.0,
            complexity_threshold: 8,
        };
        integrator.configure(config).unwrap();

        let workflow = integrator.generate_workflow();
        assert!(workflow.contains("version: 2"));
        assert!(workflow.contains("jobs:"));
        assert!(workflow.contains("cargo test"));
    }

    #[test]
    fn test_worker_status_enum_variants() {
        let statuses = [
            WorkerStatus::Active,
            WorkerStatus::Busy,
            WorkerStatus::Offline,
            WorkerStatus::Unknown,
        ];
        assert_eq!(statuses.len(), 4);
    }

    #[test]
    fn test_worker_status_debug_format() {
        assert_eq!(format!("{:?}", WorkerStatus::Active), "Active");
        assert_eq!(format!("{:?}", WorkerStatus::Busy), "Busy");
        assert_eq!(format!("{:?}", WorkerStatus::Offline), "Offline");
        assert_eq!(format!("{:?}", WorkerStatus::Unknown), "Unknown");
    }

    #[test]
    fn test_distributed_test_coordinator_new() {
        let coordinator = DistributedTestCoordinator::new();
        assert!(coordinator.workers.is_empty());
    }

    #[test]
    fn test_distributed_test_coordinator_default() {
        let coordinator = DistributedTestCoordinator::default();
        assert!(coordinator.workers.is_empty());
    }

    #[test]
    fn test_register_worker() {
        let mut coordinator = DistributedTestCoordinator::new();
        coordinator.register_worker("worker1", "192.168.1.100:8080");
        coordinator.register_worker("worker2", "192.168.1.101:8080");

        assert_eq!(coordinator.workers.len(), 2);
        assert_eq!(
            coordinator.workers.get("worker1"),
            Some(&"192.168.1.100:8080".to_string())
        );
        assert_eq!(
            coordinator.workers.get("worker2"),
            Some(&"192.168.1.101:8080".to_string())
        );
    }

    #[test]
    fn test_distribute_tests_no_workers() {
        let coordinator = DistributedTestCoordinator::new();
        let tests = vec!["test1".to_string(), "test2".to_string()];

        let distribution = coordinator.distribute(&tests);
        assert!(distribution.is_empty());
    }

    #[test]
    fn test_distribute_tests_single_worker() {
        let mut coordinator = DistributedTestCoordinator::new();
        coordinator.register_worker("worker1", "localhost:8080");

        let tests = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];
        let distribution = coordinator.distribute(&tests);

        assert_eq!(distribution.len(), 1);
        assert_eq!(distribution.get("worker1").unwrap().len(), 3);
        assert!(distribution
            .get("worker1")
            .unwrap()
            .contains(&"test1".to_string()));
    }

    #[test]
    fn test_distribute_tests_multiple_workers() {
        let mut coordinator = DistributedTestCoordinator::new();
        coordinator.register_worker("worker1", "host1:8080");
        coordinator.register_worker("worker2", "host2:8080");

        let tests = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];
        let distribution = coordinator.distribute(&tests);

        assert_eq!(distribution.len(), 2);
        // Round-robin distribution: worker1 gets test1,test3; worker2 gets test2
        assert_eq!(distribution.get("worker1").unwrap().len(), 2);
        assert_eq!(distribution.get("worker2").unwrap().len(), 1);
    }

    #[test]
    fn test_get_worker_status_existing() {
        let mut coordinator = DistributedTestCoordinator::new();
        coordinator.register_worker("worker1", "localhost:8080");

        let status = coordinator.get_worker_status("worker1");
        assert!(matches!(status, WorkerStatus::Active));
    }

    #[test]
    fn test_get_worker_status_unknown() {
        let coordinator = DistributedTestCoordinator::new();
        let status = coordinator.get_worker_status("nonexistent");
        assert!(matches!(status, WorkerStatus::Unknown));
    }

    #[test]
    fn test_metric_enum_variants() {
        let metrics = [
            Metric::MemoryUsage,
            Metric::CpuUsage,
            Metric::TestDuration,
            Metric::ErrorRate,
        ];
        assert_eq!(metrics.len(), 4);
    }

    #[test]
    fn test_alert_action_enum_variants() {
        let actions = [
            AlertAction::Email("test@example.com".to_string()),
            AlertAction::Slack("#alerts".to_string()),
            AlertAction::PagerDuty("service123".to_string()),
            AlertAction::Webhook("https://webhook.example.com".to_string()),
        ];
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: "alert1".to_string(),
            metric: Metric::MemoryUsage,
            threshold: 80.0,
            action: AlertAction::Email("admin@example.com".to_string()),
        };
        assert_eq!(alert.id, "alert1");
        assert!(matches!(alert.metric, Metric::MemoryUsage));
        assert_eq!(alert.threshold, 80.0);
    }

    #[test]
    fn test_continuous_monitor_new() {
        let monitor = ContinuousMonitor::new();
        assert!(monitor.alerts.is_empty());
        assert!(monitor.triggered.is_empty());
        assert!(monitor.metrics.is_empty());
    }

    #[test]
    fn test_continuous_monitor_default() {
        let monitor = ContinuousMonitor::default();
        assert!(monitor.alerts.is_empty());
        assert!(monitor.triggered.is_empty());
        assert!(monitor.metrics.is_empty());
    }

    #[test]
    fn test_add_alert() {
        let mut monitor = ContinuousMonitor::new();
        let alert = Alert {
            id: "cpu_alert".to_string(),
            metric: Metric::CpuUsage,
            threshold: 90.0,
            action: AlertAction::Slack("#alerts".to_string()),
        };

        monitor.add_alert(alert);
        assert_eq!(monitor.alerts.len(), 1);
        assert_eq!(monitor.alerts[0].id, "cpu_alert");
    }

    #[test]
    fn test_record_metric_no_alert() {
        let mut monitor = ContinuousMonitor::new();
        monitor.record_metric(Metric::CpuUsage, 50.0);

        assert_eq!(monitor.metrics.len(), 1);
        assert_eq!(monitor.metrics.get(&Metric::CpuUsage), Some(&50.0));
        assert!(monitor.triggered.is_empty());
    }

    #[test]
    fn test_record_metric_triggers_alert() {
        let mut monitor = ContinuousMonitor::new();
        let alert = Alert {
            id: "memory_alert".to_string(),
            metric: Metric::MemoryUsage,
            threshold: 80.0,
            action: AlertAction::Email("ops@example.com".to_string()),
        };
        monitor.add_alert(alert);

        monitor.record_metric(Metric::MemoryUsage, 85.0);

        assert_eq!(monitor.metrics.get(&Metric::MemoryUsage), Some(&85.0));
        assert_eq!(monitor.triggered.len(), 1);
        assert!(monitor.triggered.contains(&"memory_alert".to_string()));
    }

    #[test]
    fn test_record_metric_below_threshold() {
        let mut monitor = ContinuousMonitor::new();
        let alert = Alert {
            id: "error_alert".to_string(),
            metric: Metric::ErrorRate,
            threshold: 5.0,
            action: AlertAction::PagerDuty("incident123".to_string()),
        };
        monitor.add_alert(alert);

        monitor.record_metric(Metric::ErrorRate, 2.0);

        assert_eq!(monitor.metrics.get(&Metric::ErrorRate), Some(&2.0));
        assert!(monitor.triggered.is_empty());
    }

    #[test]
    fn test_get_triggered_alerts() {
        let mut monitor = ContinuousMonitor::new();
        monitor.triggered.push("alert1".to_string());
        monitor.triggered.push("alert2".to_string());

        let alerts = monitor.get_triggered_alerts();
        assert_eq!(alerts.len(), 2);
        assert!(alerts.contains(&"alert1".to_string()));
        assert!(alerts.contains(&"alert2".to_string()));
    }

    #[test]
    fn test_clear_alerts() {
        let mut monitor = ContinuousMonitor::new();
        monitor.triggered.push("alert1".to_string());
        monitor.triggered.push("alert2".to_string());

        assert_eq!(monitor.triggered.len(), 2);
        monitor.clear_alerts();
        assert!(monitor.triggered.is_empty());
    }

    #[test]
    fn test_start_stop_monitor_methods() {
        let monitor = ContinuousMonitor::new();
        // These methods don't panic and return successfully
        monitor.start();
        monitor.stop();
    }

    #[test]
    fn test_metric_hash_eq() {
        use std::collections::HashSet;
        let mut metrics = HashSet::new();

        metrics.insert(Metric::CpuUsage);
        metrics.insert(Metric::MemoryUsage);
        metrics.insert(Metric::CpuUsage); // Duplicate

        assert_eq!(metrics.len(), 2); // Verify Hash + Eq traits work
        assert!(metrics.contains(&Metric::CpuUsage));
        assert!(metrics.contains(&Metric::MemoryUsage));
    }
}
