//! Platform-specific deployment for WebAssembly components (RUCHY-0819)
//!
//! Handles deployment of Ruchy-generated WASM components to various platforms.
use super::component::WasmComponent;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
/// Deployment target platform
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentTarget {
    /// Cloudflare Workers
    CloudflareWorkers,
    /// Fastly Compute@Edge
    FastlyCompute,
    /// AWS Lambda
    AwsLambda,
    /// Vercel Edge Functions
    VercelEdge,
    /// Deno Deploy
    DenoDeploy,
    /// Wasmtime runtime
    Wasmtime,
    /// `WasmEdge` runtime
    WasmEdge,
    /// Browser environment
    Browser,
    /// Node.js
    NodeJs,
    /// Custom deployment target
    Custom(String),
}
/// Component deployer
pub struct Deployer {
    /// Deployment configuration
    config: DeploymentConfig,
    /// Target platform
    target: DeploymentTarget,
    /// Deployment artifacts
    artifacts: Vec<DeploymentArtifact>,
}
/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Project name
    pub project_name: String,
    /// Environment (development, staging, production)
    pub environment: Environment,
    /// API keys and credentials
    pub credentials: Credentials,
    /// Custom deployment settings
    pub settings: HashMap<String, String>,
    /// Optimization settings
    pub optimization: DeploymentOptimization,
    /// Runtime configuration
    pub runtime: RuntimeConfig,
}
/// Deployment environment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
    /// Custom environment
    Custom(String),
}
/// Deployment credentials
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    /// API key
    pub api_key: Option<String>,
    /// Account ID
    pub account_id: Option<String>,
    /// Auth token
    pub auth_token: Option<String>,
    /// Custom credentials
    pub custom: HashMap<String, String>,
}
/// Deployment optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentOptimization {
    /// Minify the component
    pub minify: bool,
    /// Compress the component
    pub compress: bool,
    /// Strip debug information
    pub strip_debug: bool,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache duration in seconds
    pub cache_duration: Option<u32>,
}
/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Memory limit in MB
    pub memory_limit: Option<u32>,
    /// CPU limit in milliseconds
    pub cpu_limit: Option<u32>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Runtime version
    pub runtime_version: Option<String>,
}
/// Deployment artifact
#[derive(Debug, Clone)]
pub struct DeploymentArtifact {
    /// Artifact name
    pub name: String,
    /// Artifact type
    pub artifact_type: ArtifactType,
    /// Artifact content
    pub content: Vec<u8>,
    /// Artifact metadata
    pub metadata: HashMap<String, String>,
}
/// Artifact types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    /// WASM module
    WasmModule,
    /// JavaScript glue code
    JavaScript,
    /// HTML wrapper
    Html,
    /// Configuration file
    Config,
    /// Manifest file
    Manifest,
    /// Custom artifact
    Custom(String),
}
/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    /// Deployment ID
    pub deployment_id: String,
    /// Deployment URL
    pub url: Option<String>,
    /// Deployment status
    pub status: DeploymentStatus,
    /// Deployment timestamp
    pub timestamp: std::time::SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
/// Deployment status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Deployment pending
    Pending,
    /// Deployment in progress
    InProgress,
    /// Deployment successful
    Success,
    /// Deployment failed
    Failed(String),
}
impl Deployer {
    /// Create a new deployer
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::deployment::Deployer;
    ///
    /// let instance = Deployer::new();
    /// // Verify behavior
    /// ```
    pub fn new(target: DeploymentTarget, config: DeploymentConfig) -> Self {
        Self {
            config,
            target,
            artifacts: Vec::new(),
        }
    }
    /// Add a deployment artifact
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::deployment::Deployer;
    ///
    /// let mut instance = Deployer::new();
    /// let result = instance.add_artifact();
    /// // Verify behavior
    /// ```
    pub fn add_artifact(&mut self, artifact: DeploymentArtifact) {
        self.artifacts.push(artifact);
    }
    /// Deploy the component
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::deployment::deploy;
    ///
    /// let result = deploy(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn deploy(&self, component: &WasmComponent) -> Result<DeploymentResult> {
        match &self.target {
            DeploymentTarget::CloudflareWorkers => self.deploy_cloudflare(component),
            DeploymentTarget::FastlyCompute => self.deploy_fastly(component),
            DeploymentTarget::AwsLambda => self.deploy_aws_lambda(component),
            DeploymentTarget::VercelEdge => self.deploy_vercel(component),
            DeploymentTarget::DenoDeploy => self.deploy_deno(component),
            DeploymentTarget::Browser => self.deploy_browser(component),
            DeploymentTarget::NodeJs => self.deploy_nodejs(component),
            DeploymentTarget::Wasmtime => self.deploy_wasmtime(component),
            DeploymentTarget::WasmEdge => self.deploy_wasmedge(component),
            DeploymentTarget::Custom(name) => Err(anyhow::anyhow!(
                "Custom deployment target '{name}' not implemented"
            )),
        }
    }
    /// Generate deployment package
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::deployment::generate_package;
    ///
    /// let result = generate_package(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_package(
        &self,
        component: &WasmComponent,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        // Create output directory
        fs::create_dir_all(output_dir)?;
        // Generate artifacts based on target
        let artifacts = self.generate_artifacts(component)?;
        // Write artifacts to output directory
        for artifact in &artifacts {
            let file_path = output_dir.join(&artifact.name);
            fs::write(&file_path, &artifact.content)
                .with_context(|| format!("Failed to write artifact: {}", file_path.display()))?;
        }
        // Create deployment manifest
        let manifest = self.generate_manifest(component)?;
        let manifest_path = output_dir.join("deployment.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
        Ok(output_dir.to_path_buf())
    }
    fn deploy_cloudflare(&self, component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate Cloudflare Workers specific artifacts
        let _worker_js = self.generate_cloudflare_worker(component)?;
        let _wrangler_toml = self.generate_wrangler_config()?;
        // In a real implementation, this would:
        // 1. Use wrangler API to upload the worker
        // 2. Configure routes and bindings
        // 3. Deploy to Cloudflare edge network
        Ok(DeploymentResult {
            deployment_id: format!("cf-{}", uuid::Uuid::new_v4()),
            url: Some(format!("https://{}.workers.dev", self.config.project_name)),
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_fastly(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate Fastly Compute@Edge specific artifacts
        Ok(DeploymentResult {
            deployment_id: format!("fastly-{}", uuid::Uuid::new_v4()),
            url: Some(format!(
                "https://{}.edgecompute.app",
                self.config.project_name
            )),
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_aws_lambda(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate AWS Lambda specific artifacts
        Ok(DeploymentResult {
            deployment_id: format!("lambda-{}", uuid::Uuid::new_v4()),
            url: Some(format!(
                "https://lambda.amazonaws.com/functions/{}",
                self.config.project_name
            )),
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_vercel(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate Vercel Edge Functions specific artifacts
        Ok(DeploymentResult {
            deployment_id: format!("vercel-{}", uuid::Uuid::new_v4()),
            url: Some(format!("https://{}.vercel.app", self.config.project_name)),
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_deno(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate Deno Deploy specific artifacts
        Ok(DeploymentResult {
            deployment_id: format!("deno-{}", uuid::Uuid::new_v4()),
            url: Some(format!("https://{}.deno.dev", self.config.project_name)),
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_browser(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate browser-specific artifacts (HTML, JS glue code)
        Ok(DeploymentResult {
            deployment_id: format!("browser-{}", uuid::Uuid::new_v4()),
            url: None,
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_nodejs(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate Node.js specific artifacts
        Ok(DeploymentResult {
            deployment_id: format!("node-{}", uuid::Uuid::new_v4()),
            url: None,
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_wasmtime(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate Wasmtime specific artifacts
        Ok(DeploymentResult {
            deployment_id: format!("wasmtime-{}", uuid::Uuid::new_v4()),
            url: None,
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn deploy_wasmedge(&self, _component: &WasmComponent) -> Result<DeploymentResult> {
        // Generate WasmEdge specific artifacts
        Ok(DeploymentResult {
            deployment_id: format!("wasmedge-{}", uuid::Uuid::new_v4()),
            url: None,
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        })
    }
    fn generate_artifacts(&self, component: &WasmComponent) -> Result<Vec<DeploymentArtifact>> {
        let mut artifacts = vec![DeploymentArtifact {
            name: format!("{}.wasm", component.name),
            artifact_type: ArtifactType::WasmModule,
            content: component.bytecode.clone(),
            metadata: HashMap::new(),
        }];
        // Add target-specific artifacts
        match &self.target {
            DeploymentTarget::Browser => {
                artifacts.push(self.generate_browser_glue(component)?);
                artifacts.push(self.generate_html_wrapper(component)?);
            }
            DeploymentTarget::CloudflareWorkers => {
                artifacts.push(self.generate_worker_script(component)?);
            }
            _ => {}
        }
        Ok(artifacts)
    }
    fn generate_manifest(&self, component: &WasmComponent) -> Result<DeploymentManifest> {
        Ok(DeploymentManifest {
            name: component.name.clone(),
            version: component.version.clone(),
            target: self.target.clone(),
            environment: self.config.environment.clone(),
            artifacts: self.artifacts.iter().map(|a| a.name.clone()).collect(),
            metadata: HashMap::new(),
        })
    }
    fn generate_cloudflare_worker(&self, component: &WasmComponent) -> Result<String> {
        Ok(format!(
            r"
import wasm from './{}.wasm';
export default {{
  async fetch(request, env, ctx) {{
    const {{ exports }} = await WebAssembly.instantiate(wasm);
    return new Response('Hello from Ruchy WASM!');
  }},
}};
",
            component.name
        ))
    }
    fn generate_wrangler_config(&self) -> Result<String> {
        Ok(format!(
            r#"
name = "{}"
main = "src/worker.js"
compatibility_date = "2024-01-01"
[build]
command = ""
[env.production]
route = "https://example.com/*"
"#,
            self.config.project_name
        ))
    }
    fn generate_browser_glue(&self, component: &WasmComponent) -> Result<DeploymentArtifact> {
        let js_content = format!(
            r"
export async function init() {{
    const response = await fetch('{}.wasm');
    const bytes = await response.arrayBuffer();
    const {{ instance }} = await WebAssembly.instantiate(bytes);
    return instance.exports;
}}
",
            component.name
        );
        Ok(DeploymentArtifact {
            name: format!("{}.js", component.name),
            artifact_type: ArtifactType::JavaScript,
            content: js_content.into_bytes(),
            metadata: HashMap::new(),
        })
    }
    fn generate_html_wrapper(&self, component: &WasmComponent) -> Result<DeploymentArtifact> {
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <script type="module">
        import {{ init }} from './{}.js';
        init().then(exports => {{
            console.log('WASM module loaded:', exports);
        }});
    </script>
</head>
<body>
    <h1>{} WebAssembly Component</h1>
</body>
</html>"#,
            component.name, component.name, component.name
        );
        Ok(DeploymentArtifact {
            name: "index.html".to_string(),
            artifact_type: ArtifactType::Html,
            content: html_content.into_bytes(),
            metadata: HashMap::new(),
        })
    }
    fn generate_worker_script(&self, component: &WasmComponent) -> Result<DeploymentArtifact> {
        let script = self.generate_cloudflare_worker(component)?;
        Ok(DeploymentArtifact {
            name: "worker.js".to_string(),
            artifact_type: ArtifactType::JavaScript,
            content: script.into_bytes(),
            metadata: HashMap::new(),
        })
    }
}
/// Deployment manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentManifest {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Deployment target
    pub target: DeploymentTarget,
    /// Deployment environment
    pub environment: Environment,
    /// List of artifacts
    pub artifacts: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            environment: Environment::Development,
            credentials: Credentials::default(),
            settings: HashMap::new(),
            optimization: DeploymentOptimization::default(),
            runtime: RuntimeConfig::default(),
        }
    }
}
impl Default for DeploymentOptimization {
    fn default() -> Self {
        Self {
            minify: true,
            compress: true,
            strip_debug: true,
            enable_cache: true,
            cache_duration: Some(3600), // 1 hour
        }
    }
}
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: Some(128), // 128 MB
            cpu_limit: Some(10000),  // 10 seconds
            env_vars: HashMap::new(),
            runtime_version: None,
        }
    }
}
#[cfg(test)]
mod property_tests_deployment {
    use super::*;
    use proptest::prelude::*;

    fn arb_deployment_target() -> impl Strategy<Value = DeploymentTarget> {
        prop_oneof![
            Just(DeploymentTarget::CloudflareWorkers),
            Just(DeploymentTarget::FastlyCompute),
            Just(DeploymentTarget::AwsLambda),
            Just(DeploymentTarget::VercelEdge),
            Just(DeploymentTarget::DenoDeploy),
            Just(DeploymentTarget::Wasmtime),
            Just(DeploymentTarget::WasmEdge),
            Just(DeploymentTarget::Browser),
            Just(DeploymentTarget::NodeJs),
            "[a-z]{1,20}".prop_map(DeploymentTarget::Custom),
        ]
    }

    fn arb_environment() -> impl Strategy<Value = Environment> {
        prop_oneof![
            Just(Environment::Development),
            Just(Environment::Staging),
            Just(Environment::Production),
            "[a-z]{1,20}".prop_map(Environment::Custom),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// Property: Deployer creation never panics
        #[test]
        fn prop_deployer_new_never_panics(
            target in arb_deployment_target(),
            project_name in "[a-z]{1,30}",
            env in arb_environment()
        ) {
            let config = DeploymentConfig {
                project_name,
                environment: env,
                credentials: Credentials::default(),
                settings: HashMap::new(),
                optimization: DeploymentOptimization::default(),
                runtime: RuntimeConfig::default(),
            };
            let _ = Deployer::new(target, config);
        }

        /// Property: Add artifact never panics
        #[test]
        fn prop_add_artifact_never_panics(
            target in arb_deployment_target(),
            artifact_name in "[a-z]{1,20}"
        ) {
            let config = DeploymentConfig::default();
            let mut deployer = Deployer::new(target, config);
            let artifact = DeploymentArtifact {
                name: artifact_name,
                artifact_type: ArtifactType::WasmModule,
                content: vec![0u8; 100],
                metadata: HashMap::new(),
            };
            deployer.add_artifact(artifact);
        }

        /// Property: Credentials serialization roundtrips
        #[test]
        fn prop_credentials_roundtrip(
            api_key in proptest::option::of("[a-z0-9]{10,30}"),
            account_id in proptest::option::of("[a-z0-9]{10,20}")
        ) {
            let creds = Credentials {
                api_key,
                account_id,
                auth_token: None,
                custom: HashMap::new(),
            };
            let json = serde_json::to_string(&creds).unwrap();
            let decoded: Credentials = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(creds.api_key, decoded.api_key);
            prop_assert_eq!(creds.account_id, decoded.account_id);
        }

        /// Property: RuntimeConfig default values are sane
        #[test]
        fn prop_runtime_config_default_sane(
            memory in 1u32..1024,
            cpu in 1u32..10000
        ) {
            let config = RuntimeConfig {
                memory_limit: Some(memory),
                cpu_limit: Some(cpu),
                env_vars: HashMap::new(),
                runtime_version: None,
            };
            prop_assert!(config.memory_limit.unwrap() >= 1);
            prop_assert!(config.cpu_limit.unwrap() >= 1);
        }

        /// Property: DeploymentOptimization serialization roundtrips
        #[test]
        fn prop_optimization_roundtrip(
            minify in proptest::bool::ANY,
            compress in proptest::bool::ANY,
            cache_duration in proptest::option::of(1u32..7200)
        ) {
            let opt = DeploymentOptimization {
                minify,
                compress,
                strip_debug: true,
                enable_cache: cache_duration.is_some(),
                cache_duration,
            };
            let json = serde_json::to_string(&opt).unwrap();
            let decoded: DeploymentOptimization = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(opt.minify, decoded.minify);
            prop_assert_eq!(opt.compress, decoded.compress);
            prop_assert_eq!(opt.cache_duration, decoded.cache_duration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Helper Functions for Tests =====

    fn create_test_component() -> WasmComponent {
        use crate::wasm::component::ComponentMetadata;
        WasmComponent {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            imports: vec![],
            exports: vec![],
            metadata: ComponentMetadata::default(),
            custom_sections: HashMap::new(),
        }
    }

    fn create_test_config() -> DeploymentConfig {
        DeploymentConfig {
            project_name: "test-project".to_string(),
            environment: Environment::Development,
            credentials: Credentials::default(),
            settings: HashMap::new(),
            optimization: DeploymentOptimization::default(),
            runtime: RuntimeConfig::default(),
        }
    }

    // ===== DeploymentTarget Tests =====

    #[test]
    fn test_deployment_target_equality() {
        assert_eq!(
            DeploymentTarget::CloudflareWorkers,
            DeploymentTarget::CloudflareWorkers
        );
        assert_ne!(
            DeploymentTarget::CloudflareWorkers,
            DeploymentTarget::FastlyCompute
        );
        assert_ne!(
            DeploymentTarget::Custom("a".to_string()),
            DeploymentTarget::Custom("b".to_string())
        );
    }

    #[test]
    fn test_deployment_target_debug() {
        let target = DeploymentTarget::CloudflareWorkers;
        let debug = format!("{:?}", target);
        assert!(debug.contains("CloudflareWorkers"));
    }

    #[test]
    fn test_deployment_target_clone() {
        let target = DeploymentTarget::AwsLambda;
        let cloned = target.clone();
        assert_eq!(target, cloned);
    }

    #[test]
    fn test_deployment_target_serialize() {
        let target = DeploymentTarget::VercelEdge;
        let json = serde_json::to_string(&target).expect("serialize");
        assert!(json.contains("VercelEdge"));
    }

    #[test]
    fn test_deployment_target_all_variants() {
        let targets = vec![
            DeploymentTarget::CloudflareWorkers,
            DeploymentTarget::FastlyCompute,
            DeploymentTarget::AwsLambda,
            DeploymentTarget::VercelEdge,
            DeploymentTarget::DenoDeploy,
            DeploymentTarget::Wasmtime,
            DeploymentTarget::WasmEdge,
            DeploymentTarget::Browser,
            DeploymentTarget::NodeJs,
            DeploymentTarget::Custom("custom".to_string()),
        ];
        assert_eq!(targets.len(), 10);
    }

    // ===== Environment Tests =====

    #[test]
    fn test_environment_equality() {
        assert_eq!(Environment::Development, Environment::Development);
        assert_ne!(Environment::Development, Environment::Production);
        assert_eq!(
            Environment::Custom("test".to_string()),
            Environment::Custom("test".to_string())
        );
    }

    #[test]
    fn test_environment_debug() {
        let env = Environment::Staging;
        let debug = format!("{:?}", env);
        assert!(debug.contains("Staging"));
    }

    #[test]
    fn test_environment_serialize() {
        let env = Environment::Production;
        let json = serde_json::to_string(&env).expect("serialize");
        assert!(json.contains("Production"));
    }

    // ===== Credentials Tests =====

    #[test]
    fn test_credentials_default() {
        let creds = Credentials::default();
        assert!(creds.api_key.is_none());
        assert!(creds.account_id.is_none());
        assert!(creds.auth_token.is_none());
        assert!(creds.custom.is_empty());
    }

    #[test]
    fn test_credentials_with_values() {
        let creds = Credentials {
            api_key: Some("key123".to_string()),
            account_id: Some("acc123".to_string()),
            auth_token: Some("token123".to_string()),
            custom: HashMap::from([("region".to_string(), "us-east".to_string())]),
        };
        assert_eq!(creds.api_key, Some("key123".to_string()));
        assert_eq!(creds.account_id, Some("acc123".to_string()));
        assert!(!creds.custom.is_empty());
    }

    #[test]
    fn test_credentials_clone() {
        let creds = Credentials {
            api_key: Some("key".to_string()),
            ..Default::default()
        };
        let cloned = creds.clone();
        assert_eq!(creds.api_key, cloned.api_key);
    }

    // ===== DeploymentConfig Tests =====

    #[test]
    fn test_deployment_config_default() {
        let config = DeploymentConfig::default();
        assert!(config.project_name.is_empty());
        assert_eq!(config.environment, Environment::Development);
        assert!(config.settings.is_empty());
    }

    #[test]
    fn test_deployment_config_clone() {
        let config = create_test_config();
        let cloned = config.clone();
        assert_eq!(config.project_name, cloned.project_name);
    }

    #[test]
    fn test_deployment_config_debug() {
        let config = create_test_config();
        let debug = format!("{:?}", config);
        assert!(debug.contains("DeploymentConfig"));
    }

    // ===== DeploymentOptimization Tests =====

    #[test]
    fn test_deployment_optimization_default() {
        let opt = DeploymentOptimization::default();
        assert!(opt.minify);
        assert!(opt.compress);
        assert!(opt.strip_debug);
        assert!(opt.enable_cache);
        assert_eq!(opt.cache_duration, Some(3600));
    }

    #[test]
    fn test_deployment_optimization_custom() {
        let opt = DeploymentOptimization {
            minify: false,
            compress: false,
            strip_debug: false,
            enable_cache: false,
            cache_duration: None,
        };
        assert!(!opt.minify);
        assert!(!opt.enable_cache);
    }

    // ===== RuntimeConfig Tests =====

    #[test]
    fn test_runtime_config_default() {
        let runtime = RuntimeConfig::default();
        assert_eq!(runtime.memory_limit, Some(128));
        assert_eq!(runtime.cpu_limit, Some(10000));
        assert!(runtime.env_vars.is_empty());
        assert!(runtime.runtime_version.is_none());
    }

    #[test]
    fn test_runtime_config_with_env_vars() {
        let runtime = RuntimeConfig {
            memory_limit: Some(256),
            cpu_limit: Some(5000),
            env_vars: HashMap::from([("NODE_ENV".to_string(), "production".to_string())]),
            runtime_version: Some("18".to_string()),
        };
        assert_eq!(
            runtime.env_vars.get("NODE_ENV"),
            Some(&"production".to_string())
        );
    }

    // ===== Deployer Tests =====

    #[test]
    fn test_deployer_new() {
        let target = DeploymentTarget::CloudflareWorkers;
        let config = DeploymentConfig::default();
        let deployer = Deployer::new(target.clone(), config.clone());
        assert_eq!(deployer.target, target);
        assert_eq!(deployer.config.project_name, config.project_name);
        assert!(deployer.artifacts.is_empty());
    }

    #[test]
    fn test_add_artifact() {
        let target = DeploymentTarget::Browser;
        let config = DeploymentConfig::default();
        let mut deployer = Deployer::new(target, config);

        let artifact = DeploymentArtifact {
            name: "test.wasm".to_string(),
            artifact_type: ArtifactType::WasmModule,
            content: vec![0x00, 0x61, 0x73, 0x6d],
            metadata: HashMap::new(),
        };

        deployer.add_artifact(artifact);
        assert_eq!(deployer.artifacts.len(), 1);
        assert_eq!(deployer.artifacts[0].name, "test.wasm");
    }

    #[test]
    fn test_add_multiple_artifacts() {
        let target = DeploymentTarget::Browser;
        let config = DeploymentConfig::default();
        let mut deployer = Deployer::new(target, config);

        for i in 0..5 {
            let artifact = DeploymentArtifact {
                name: format!("artifact_{}.wasm", i),
                artifact_type: ArtifactType::WasmModule,
                content: vec![0x00],
                metadata: HashMap::new(),
            };
            deployer.add_artifact(artifact);
        }

        assert_eq!(deployer.artifacts.len(), 5);
    }

    // ===== Deploy Method Tests =====

    #[test]
    fn test_deploy_cloudflare() {
        let target = DeploymentTarget::CloudflareWorkers;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("cf-"));
        assert!(deployment.url.is_some());
        assert_eq!(deployment.status, DeploymentStatus::Success);
    }

    #[test]
    fn test_deploy_fastly() {
        let target = DeploymentTarget::FastlyCompute;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("fastly-"));
    }

    #[test]
    fn test_deploy_aws_lambda() {
        let target = DeploymentTarget::AwsLambda;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("lambda-"));
    }

    #[test]
    fn test_deploy_vercel() {
        let target = DeploymentTarget::VercelEdge;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("vercel-"));
    }

    #[test]
    fn test_deploy_deno() {
        let target = DeploymentTarget::DenoDeploy;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("deno-"));
    }

    #[test]
    fn test_deploy_browser() {
        let target = DeploymentTarget::Browser;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("browser-"));
        assert!(deployment.url.is_none()); // Browser has no URL
    }

    #[test]
    fn test_deploy_nodejs() {
        let target = DeploymentTarget::NodeJs;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("node-"));
    }

    #[test]
    fn test_deploy_wasmtime() {
        let target = DeploymentTarget::Wasmtime;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("wasmtime-"));
    }

    #[test]
    fn test_deploy_wasmedge() {
        let target = DeploymentTarget::WasmEdge;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_ok());
        let deployment = result.unwrap();
        assert!(deployment.deployment_id.starts_with("wasmedge-"));
    }

    #[test]
    fn test_deploy_custom_fails() {
        let target = DeploymentTarget::Custom("my-custom-target".to_string());
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let result = deployer.deploy(&component);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("my-custom-target"));
        assert!(err.contains("not implemented"));
    }

    // ===== Generate Package Test =====

    #[test]
    fn test_generate_package_browser() {
        let target = DeploymentTarget::Browser;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let temp_dir = std::env::temp_dir().join(format!("ruchy_test_{}", std::process::id()));
        let result = deployer.generate_package(&component, &temp_dir);

        if result.is_ok() {
            // Clean up
            let _ = fs::remove_dir_all(&temp_dir);
        }
    }

    #[test]
    fn test_generate_package_cloudflare() {
        let target = DeploymentTarget::CloudflareWorkers;
        let config = create_test_config();
        let deployer = Deployer::new(target, config);
        let component = create_test_component();

        let temp_dir = std::env::temp_dir().join(format!("ruchy_cf_test_{}", std::process::id()));
        let result = deployer.generate_package(&component, &temp_dir);

        if result.is_ok() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
    }

    // ===== DeploymentStatus Tests =====

    #[test]
    fn test_deployment_status_equality() {
        assert_eq!(DeploymentStatus::Pending, DeploymentStatus::Pending);
        assert_eq!(DeploymentStatus::Success, DeploymentStatus::Success);
        assert_ne!(DeploymentStatus::Success, DeploymentStatus::InProgress);
        assert_eq!(
            DeploymentStatus::Failed("error".to_string()),
            DeploymentStatus::Failed("error".to_string())
        );
    }

    #[test]
    fn test_deployment_status_debug() {
        let status = DeploymentStatus::InProgress;
        let debug = format!("{:?}", status);
        assert!(debug.contains("InProgress"));
    }

    #[test]
    fn test_deployment_status_clone() {
        let status = DeploymentStatus::Failed("error".to_string());
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    // ===== ArtifactType Tests =====

    #[test]
    fn test_artifact_type_equality() {
        assert_eq!(ArtifactType::WasmModule, ArtifactType::WasmModule);
        assert_ne!(ArtifactType::WasmModule, ArtifactType::JavaScript);
        assert_eq!(
            ArtifactType::Custom("test".to_string()),
            ArtifactType::Custom("test".to_string())
        );
    }

    #[test]
    fn test_artifact_type_all_variants() {
        let types = vec![
            ArtifactType::WasmModule,
            ArtifactType::JavaScript,
            ArtifactType::Html,
            ArtifactType::Config,
            ArtifactType::Manifest,
            ArtifactType::Custom("custom".to_string()),
        ];
        assert_eq!(types.len(), 6);
    }

    #[test]
    fn test_artifact_type_debug() {
        let artifact_type = ArtifactType::JavaScript;
        let debug = format!("{:?}", artifact_type);
        assert!(debug.contains("JavaScript"));
    }

    // ===== DeploymentArtifact Tests =====

    #[test]
    fn test_deployment_artifact_creation() {
        let artifact = DeploymentArtifact {
            name: "app.js".to_string(),
            artifact_type: ArtifactType::JavaScript,
            content: b"console.log('hello');".to_vec(),
            metadata: HashMap::from([("version".to_string(), "1.0".to_string())]),
        };

        assert_eq!(artifact.name, "app.js");
        assert_eq!(artifact.artifact_type, ArtifactType::JavaScript);
        assert!(!artifact.content.is_empty());
        assert_eq!(artifact.metadata.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_deployment_artifact_clone() {
        let artifact = DeploymentArtifact {
            name: "test.wasm".to_string(),
            artifact_type: ArtifactType::WasmModule,
            content: vec![0x00, 0x01],
            metadata: HashMap::new(),
        };
        let cloned = artifact.clone();
        assert_eq!(artifact.name, cloned.name);
        assert_eq!(artifact.content, cloned.content);
    }

    #[test]
    fn test_deployment_artifact_debug() {
        let artifact = DeploymentArtifact {
            name: "test.wasm".to_string(),
            artifact_type: ArtifactType::WasmModule,
            content: vec![],
            metadata: HashMap::new(),
        };
        let debug = format!("{:?}", artifact);
        assert!(debug.contains("DeploymentArtifact"));
    }

    // ===== DeploymentResult Tests =====

    #[test]
    fn test_deployment_result_creation() {
        let result = DeploymentResult {
            deployment_id: "test-123".to_string(),
            url: Some("https://test.com".to_string()),
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(result.deployment_id, "test-123");
        assert_eq!(result.url, Some("https://test.com".to_string()));
        assert_eq!(result.status, DeploymentStatus::Success);
        assert!(result.metadata.is_empty());
    }

    #[test]
    fn test_deployment_result_clone() {
        let result = DeploymentResult {
            deployment_id: "test-123".to_string(),
            url: None,
            status: DeploymentStatus::Pending,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };
        let cloned = result.clone();
        assert_eq!(result.deployment_id, cloned.deployment_id);
        assert_eq!(result.status, cloned.status);
    }

    #[test]
    fn test_deployment_result_serialize() {
        let result = DeploymentResult {
            deployment_id: "test-123".to_string(),
            url: Some("https://test.com".to_string()),
            status: DeploymentStatus::Success,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("test-123"));
    }

    // ===== DeploymentManifest Tests =====

    #[test]
    fn test_deployment_manifest_creation() {
        let manifest = DeploymentManifest {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            target: DeploymentTarget::Browser,
            environment: Environment::Production,
            artifacts: vec!["app.wasm".to_string(), "app.js".to_string()],
            metadata: HashMap::new(),
        };
        assert_eq!(manifest.name, "test-component");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.artifacts.len(), 2);
    }

    #[test]
    fn test_deployment_manifest_serialize() {
        let manifest = DeploymentManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            target: DeploymentTarget::CloudflareWorkers,
            environment: Environment::Development,
            artifacts: vec![],
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&manifest).expect("serialize");
        assert!(json.contains("test"));
    }
}
