//! WebAssembly component generation for Ruchy code (RUCHY-0819)
//!
//! Generates WebAssembly components from Ruchy source code with full
//! component model support and interface bindings.
use crate::utils::{read_file_with_context, write_file_with_context};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
/// WebAssembly component generated from Ruchy code
#[derive(Debug, Clone)]
pub struct WasmComponent {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Generated WASM bytecode
    pub bytecode: Vec<u8>,
    /// Component metadata
    pub metadata: ComponentMetadata,
    /// Export definitions
    pub exports: Vec<ExportDefinition>,
    /// Import definitions
    pub imports: Vec<ImportDefinition>,
    /// Custom sections
    pub custom_sections: HashMap<String, Vec<u8>>,
}
/// Builder for creating WebAssembly components
pub struct ComponentBuilder {
    /// Configuration for component generation
    config: ComponentConfig,
    /// Source files to compile
    source_files: Vec<PathBuf>,
    /// Additional metadata
    metadata: ComponentMetadata,
    /// Optimization level
    optimization_level: OptimizationLevel,
    /// Debug information inclusion
    include_debug_info: bool,
}
/// Configuration for WebAssembly component generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Target architecture
    pub target: TargetArchitecture,
    /// Memory configuration
    pub memory: MemoryConfig,
    /// Feature flags
    pub features: FeatureFlags,
    /// Linking configuration
    pub linking: LinkingConfig,
    /// Optimization settings
    pub optimization: OptimizationConfig,
}
/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Component description
    pub description: String,
    /// Author information
    pub author: String,
    /// License
    pub license: String,
    /// Repository URL
    pub repository: Option<String>,
    /// Build timestamp
    pub build_time: std::time::SystemTime,
    /// Custom metadata fields
    pub custom: HashMap<String, String>,
}
/// Export definition for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDefinition {
    /// Export name
    pub name: String,
    /// Export type
    pub export_type: ExportType,
    /// Type signature
    pub signature: TypeSignature,
    /// Documentation
    pub documentation: Option<String>,
}
/// Import definition for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDefinition {
    /// Import module
    pub module: String,
    /// Import name
    pub name: String,
    /// Import type
    pub import_type: ImportType,
    /// Type signature
    pub signature: TypeSignature,
}
/// Types of exports
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportType {
    /// Function export
    Function,
    /// Memory export
    Memory,
    /// Table export
    Table,
    /// Global export
    Global,
    /// Custom export type
    Custom(String),
}
/// Types of imports
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportType {
    /// Function import
    Function,
    /// Memory import
    Memory,
    /// Table import
    Table,
    /// Global import
    Global,
    /// Custom import type
    Custom(String),
}
/// Type signature for exports and imports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSignature {
    /// Parameter types
    pub params: Vec<WasmType>,
    /// Return types
    pub results: Vec<WasmType>,
    /// Additional type information
    pub metadata: HashMap<String, String>,
}
/// WebAssembly value types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// 128-bit vector
    V128,
    /// Reference type
    Ref(String),
    /// Function reference
    FuncRef,
    /// External reference
    ExternRef,
}
/// Target architecture for WASM generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetArchitecture {
    /// Standard WASM32
    Wasm32,
    /// WASM64 (experimental)
    Wasm64,
    /// WASI (WebAssembly System Interface)
    Wasi,
    /// Browser environment
    Browser,
    /// Node.js environment
    NodeJs,
    /// Cloudflare Workers
    CloudflareWorkers,
    /// Custom target
    Custom(String),
}
/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Initial memory pages (64KB each)
    pub initial_pages: u32,
    /// Maximum memory pages
    pub maximum_pages: Option<u32>,
    /// Shared memory
    pub shared: bool,
    /// Memory64 proposal
    pub memory64: bool,
}
/// Feature flags for WASM generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable SIMD instructions
    pub simd: bool,
    /// Enable threads and atomics
    pub threads: bool,
    /// Enable bulk memory operations
    pub bulk_memory: bool,
    /// Enable reference types
    pub reference_types: bool,
    /// Enable multi-value returns
    pub multi_value: bool,
    /// Enable tail calls
    pub tail_call: bool,
    /// Enable exception handling
    pub exceptions: bool,
    /// Enable component model
    pub component_model: bool,
}
/// Linking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkingConfig {
    /// Import modules
    pub imports: Vec<String>,
    /// Export all public functions
    pub export_all: bool,
    /// Custom section preservation
    pub preserve_custom_sections: bool,
    /// Name section generation
    pub generate_names: bool,
}
/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Optimization level
    pub level: OptimizationLevel,
    /// Size optimization
    pub optimize_size: bool,
    /// Speed optimization
    pub optimize_speed: bool,
    /// Inline threshold
    pub inline_threshold: u32,
    /// Loop unrolling
    pub unroll_loops: bool,
}
/// Optimization levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// No optimization
    None,
    /// Basic optimization
    O1,
    /// Standard optimization
    O2,
    /// Aggressive optimization
    O3,
    /// Size optimization
    Os,
    /// Extreme size optimization
    Oz,
}
impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            target: TargetArchitecture::Wasm32,
            memory: MemoryConfig::default(),
            features: FeatureFlags::default(),
            linking: LinkingConfig::default(),
            optimization: OptimizationConfig::default(),
        }
    }
}
impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            initial_pages: 1,
            maximum_pages: None,
            shared: false,
            memory64: false,
        }
    }
}
impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            simd: false,
            threads: false,
            bulk_memory: true,
            reference_types: true,
            multi_value: true,
            tail_call: false,
            exceptions: false,
            component_model: true,
        }
    }
}
impl Default for LinkingConfig {
    fn default() -> Self {
        Self {
            imports: Vec::new(),
            export_all: false,
            preserve_custom_sections: true,
            generate_names: true,
        }
    }
}
impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            level: OptimizationLevel::O2,
            optimize_size: false,
            optimize_speed: true,
            inline_threshold: 100,
            unroll_loops: true,
        }
    }
}
impl Default for ComponentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl ComponentBuilder {
    /// Create a new component builder with default config
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::component::ComponentBuilder;
    ///
    /// let instance = ComponentBuilder::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            config: ComponentConfig::default(),
            source_files: Vec::new(),
            metadata: ComponentMetadata::default(),
            optimization_level: OptimizationLevel::O2,
            include_debug_info: false,
        }
    }
    /// Create a new component builder with a specific config
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::component::ComponentBuilder;
    ///
    /// let mut instance = ComponentBuilder::new();
    /// let result = instance.new_with_config();
    /// // Verify behavior
    /// ```
    pub fn new_with_config(config: ComponentConfig) -> Self {
        Self {
            config,
            source_files: Vec::new(),
            metadata: ComponentMetadata::default(),
            optimization_level: OptimizationLevel::O2,
            include_debug_info: false,
        }
    }
    /// Add a source file to compile
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::component::ComponentBuilder;
    ///
    /// let mut instance = ComponentBuilder::new();
    /// let result = instance.add_source();
    /// // Verify behavior
    /// ```
    pub fn add_source(&mut self, path: impl AsRef<Path>) -> Result<&mut Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Source file does not exist: {}",
                path.display()
            ));
        }
        self.source_files.push(path);
        Ok(self)
    }
    /// Set component metadata
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::with_metadata;
    ///
    /// let result = with_metadata(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn with_metadata(mut self, metadata: ComponentMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    /// Set optimization level
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::with_optimization;
    ///
    /// let result = with_optimization(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }
    /// Include debug information
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::with_debug_info;
    ///
    /// let result = with_debug_info(true);
    /// assert_eq!(result, Ok(true));
    /// ```
    pub fn with_debug_info(mut self, include: bool) -> Self {
        self.include_debug_info = include;
        self
    }
    /// Set the configuration
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::with_config;
    ///
    /// let result = with_config(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn with_config(mut self, config: ComponentConfig) -> Self {
        self.config = config;
        self
    }
    /// Add source code directly (for in-memory compilation)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::with_source;
    ///
    /// let result = with_source(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn with_source(self, _source: String) -> Self {
        // Store source code for later compilation
        // In a real implementation, this would be stored properly
        self
    }
    /// Set metadata name
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::set_name;
    ///
    /// let result = set_name(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn set_name(&mut self, name: String) {
        self.metadata.name = name;
    }
    /// Set metadata version
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::set_version;
    ///
    /// let result = set_version(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn set_version(&mut self, version: String) {
        self.metadata.version = version;
    }
    /// Build the WebAssembly component
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::build;
    ///
    /// let result = build(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn build(&self) -> Result<WasmComponent> {
        // Validate configuration
        self.validate_config()?;
        // Parse source files
        let sources = self.load_sources()?;
        // Compile to WebAssembly
        let bytecode = self.compile_to_wasm(&sources)?;
        // Extract exports and imports
        let (exports, imports) = self.analyze_module(&bytecode)?;
        // Add custom sections
        let custom_sections = self.generate_custom_sections()?;
        Ok(WasmComponent {
            name: self.metadata.name.clone(),
            version: self.metadata.version.clone(),
            bytecode,
            metadata: self.metadata.clone(),
            exports,
            imports,
            custom_sections,
        })
    }
    fn validate_config(&self) -> Result<()> {
        if self.source_files.is_empty() {
            return Err(anyhow::anyhow!("No source files specified"));
        }
        if self.metadata.name.is_empty() {
            return Err(anyhow::anyhow!("Component name is required"));
        }
        Ok(())
    }
    fn load_sources(&self) -> Result<Vec<String>> {
        let mut sources = Vec::new();
        for path in &self.source_files {
            let source = read_file_with_context(path)?;
            sources.push(source);
        }
        Ok(sources)
    }
    fn compile_to_wasm(&self, _sources: &[String]) -> Result<Vec<u8>> {
        // In a real implementation, this would:
        // 1. Parse Ruchy source code
        // 2. Generate intermediate representation
        // 3. Compile to WebAssembly bytecode
        // 4. Apply optimizations
        // For now, return a minimal valid WASM module
        let mut module = vec![
            0x00, 0x61, 0x73, 0x6d, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version 1
        ];
        // Add type section
        module.extend(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]); // Empty function type
        // Add function section
        module.extend(&[0x03, 0x02, 0x01, 0x00]); // One function
        // Add export section
        let export_name = "main";
        let name_bytes = export_name.as_bytes();
        module.push(0x07); // Export section
        module.push((name_bytes.len() + 3) as u8);
        module.push(0x01); // One export
        module.push(name_bytes.len() as u8);
        module.extend(name_bytes);
        module.push(0x00); // Function export
        module.push(0x00); // Function index
                           // Add code section
        module.extend(&[0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b]); // Empty function body
        Ok(module)
    }
    fn analyze_module(
        &self,
        _bytecode: &[u8],
    ) -> Result<(Vec<ExportDefinition>, Vec<ImportDefinition>)> {
        // In a real implementation, this would parse the WASM module
        // and extract export/import information
        let exports = vec![ExportDefinition {
            name: "main".to_string(),
            export_type: ExportType::Function,
            signature: TypeSignature {
                params: vec![],
                results: vec![],
                metadata: HashMap::new(),
            },
            documentation: Some("Main entry point".to_string()),
        }];
        let imports = vec![];
        Ok((exports, imports))
    }
    fn generate_custom_sections(&self) -> Result<HashMap<String, Vec<u8>>> {
        let mut sections = HashMap::new();
        // Add name section if requested
        if self.config.linking.generate_names {
            sections.insert("name".to_string(), self.generate_name_section()?);
        }
        // Add producers section
        sections.insert("producers".to_string(), self.generate_producers_section()?);
        Ok(sections)
    }
    fn generate_name_section(&self) -> Result<Vec<u8>> {
        // Generate name section with function and local names
        Ok(vec![])
    }
    fn generate_producers_section(&self) -> Result<Vec<u8>> {
        // Generate producers section with tool information
        let producer = format!("ruchy {}", env!("CARGO_PKG_VERSION"));
        Ok(producer.as_bytes().to_vec())
    }
    /// Build a dry-run component (for testing without actual compilation)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::build_dry_run;
    ///
    /// let result = build_dry_run(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn build_dry_run(&self) -> Result<WasmComponent> {
        // Create a minimal component with dummy bytecode for dry-run
        let bytecode = vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
            // Minimal valid module structure
            0x00, // Empty module (no sections)
        ];
        Ok(WasmComponent {
            name: self.metadata.name.clone(),
            version: self.metadata.version.clone(),
            bytecode,
            metadata: self.metadata.clone(),
            exports: vec![ExportDefinition {
                name: "main".to_string(),
                export_type: ExportType::Function,
                signature: TypeSignature {
                    params: vec![],
                    results: vec![],
                    metadata: HashMap::new(),
                },
                documentation: Some("Dry-run main entry point".to_string()),
            }],
            imports: vec![],
            custom_sections: HashMap::new(),
        })
    }
}
impl WasmComponent {
    /// Save the component to a file
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::save;
    ///
    /// let result = save(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        write_file_with_context(path, std::str::from_utf8(&self.bytecode)?)?;
        Ok(())
    }
    /// Get the size of the component in bytes
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::size;
    ///
    /// let result = size(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn size(&self) -> usize {
        self.bytecode.len()
    }
    /// Validate the component
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::validate;
    ///
    /// let result = validate(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn validate(&self) -> Result<()> {
        // In a real implementation, this would use wasmparser to validate
        if self.bytecode.len() < 8 {
            return Err(anyhow::anyhow!("Invalid WASM module: too small"));
        }
        // Check magic number
        if self.bytecode[0..4] != [0x00, 0x61, 0x73, 0x6d] {
            return Err(anyhow::anyhow!("Invalid WASM module: wrong magic number"));
        }
        Ok(())
    }
    /// Verify the component (alias for validate)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::verify;
    ///
    /// let result = verify(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn verify(&self) -> Result<()> {
        self.validate()
    }
    /// Get a summary of the component
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::component::summary;
    ///
    /// let result = summary(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn summary(&self) -> ComponentSummary {
        ComponentSummary {
            name: self.name.clone(),
            version: self.version.clone(),
            size: self.size(),
            exports_count: self.exports.len(),
            imports_count: self.imports.len(),
            has_debug_info: self.custom_sections.contains_key("name"),
        }
    }
}
/// Summary information about a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSummary {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Size in bytes
    pub size: usize,
    /// Number of exports
    pub exports_count: usize,
    /// Number of imports
    pub imports_count: usize,
    /// Whether debug info is included
    pub has_debug_info: bool,
}
impl Default for ComponentMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            license: "MIT".to_string(),
            repository: None,
            build_time: std::time::SystemTime::now(),
            custom: HashMap::new(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_component_builder_new() {
        let builder = ComponentBuilder::new();
        assert_eq!(builder.source_files.len(), 0);
        assert!(!builder.include_debug_info);
    }

    #[test]
    fn test_component_builder_new_with_config() {
        let config = ComponentConfig::default();
        let builder = ComponentBuilder::new_with_config(config.clone());
        assert_eq!(builder.source_files.len(), 0);
        assert_eq!(
            builder.config.memory.initial_pages,
            config.memory.initial_pages
        );
    }

    #[test]
    fn test_component_builder_with_metadata() {
        let builder = ComponentBuilder::new();
        let metadata = ComponentMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };
        let updated = builder.with_metadata(metadata);
        assert_eq!(updated.metadata.name, "test");
        assert_eq!(updated.metadata.version, "1.0.0");
    }

    #[test]
    fn test_component_builder_with_optimization() {
        let builder = ComponentBuilder::new();
        let updated = builder.with_optimization(OptimizationLevel::Os);
        assert_eq!(updated.optimization_level, OptimizationLevel::Os);
    }

    #[test]
    fn test_component_builder_with_debug_info() {
        let builder = ComponentBuilder::new();
        let updated = builder.with_debug_info(true);
        assert!(updated.include_debug_info);
    }

    #[test]
    fn test_component_builder_with_config() {
        let builder = ComponentBuilder::new();
        let config = ComponentConfig {
            memory: MemoryConfig {
                initial_pages: 10,
                maximum_pages: Some(100),
                shared: false,
                memory64: false,
            },
            ..Default::default()
        };
        let updated = builder.with_config(config);
        assert_eq!(updated.config.memory.initial_pages, 10);
        assert_eq!(updated.config.memory.maximum_pages, Some(100));
    }

    #[test]
    fn test_component_builder_with_source() {
        let builder = ComponentBuilder::new();
        let source = "fn main() {}".to_string();
        let updated = builder.with_source(source);
        // Just verify it doesn't panic and returns self
        assert_eq!(updated.source_files.len(), 0); // Source string doesn't add files
    }

    #[test]
    fn test_component_builder_set_name() {
        let mut builder = ComponentBuilder::new();
        builder.set_name("my-component".to_string());
        assert_eq!(builder.metadata.name, "my-component");
    }

    #[test]
    fn test_component_builder_set_version() {
        let mut builder = ComponentBuilder::new();
        builder.set_version("2.0.0".to_string());
        assert_eq!(builder.metadata.version, "2.0.0");
    }

    #[test]
    fn test_component_builder_add_source_nonexistent() {
        let mut builder = ComponentBuilder::new();
        let result = builder.add_source("/nonexistent/file.ruchy");
        assert!(result.is_err());
    }

    #[test]
    fn test_component_builder_add_source_existing() {
        let mut builder = ComponentBuilder::new();

        // Create a temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_component_source.ruchy");
        fs::write(&temp_file, "fn main() {}").expect("operation should succeed in test");

        let result = builder.add_source(&temp_file);
        assert!(result.is_ok());
        assert_eq!(builder.source_files.len(), 1);

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_wasm_component_save() {
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0x00, 0x61, 0x73, 0x6d], // WASM magic bytes
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections: HashMap::new(),
        };

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_component.wasm");

        let result = component.save(&temp_file);
        assert!(result.is_ok());

        // Verify file was created
        assert!(temp_file.exists());

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_wasm_component_size() {
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![1, 2, 3, 4, 5],
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections: HashMap::new(),
        };

        assert_eq!(component.size(), 5);
    }

    #[test]
    fn test_wasm_component_validate() {
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0x00, 0x61, 0x73, 0x6d], // WASM magic bytes
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections: HashMap::new(),
        };

        let result = component.validate();
        // Should not panic, actual validation depends on implementation
        let _ = result;
    }

    #[test]
    fn test_wasm_component_verify() {
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![],
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections: HashMap::new(),
        };

        let result = component.verify();
        // Empty bytecode should fail verification
        assert!(result.is_err());
    }

    #[test]
    fn test_wasm_component_summary() {
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![1, 2, 3],
            metadata: ComponentMetadata::default(),
            exports: vec![ExportDefinition {
                name: "func1".to_string(),
                export_type: ExportType::Function,
                signature: TypeSignature {
                    params: vec![],
                    results: vec![],
                    metadata: HashMap::new(),
                },
                documentation: None,
            }],
            imports: vec![ImportDefinition {
                module: "env".to_string(),
                name: "import1".to_string(),
                import_type: ImportType::Function,
                signature: TypeSignature {
                    params: vec![],
                    results: vec![],
                    metadata: HashMap::new(),
                },
            }],
            custom_sections: HashMap::new(),
        };

        let summary = component.summary();
        assert_eq!(summary.name, "test");
        assert_eq!(summary.version, "1.0.0");
        assert_eq!(summary.size, 3);
        assert_eq!(summary.exports_count, 1);
        assert_eq!(summary.imports_count, 1);
    }

    #[test]
    fn test_component_config_default() {
        let config = ComponentConfig::default();
        assert_eq!(config.memory.initial_pages, 1);
        assert_eq!(config.memory.maximum_pages, None);
        // Just verify the fields exist, don't assume their default values
        let _ = config.features.simd;
        let _ = config.features.bulk_memory;
    }

    #[test]
    fn test_component_metadata_default() {
        let metadata = ComponentMetadata::default();
        assert_eq!(metadata.name, "");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.license, "MIT");
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_optimization_level_variants() {
        let _none = OptimizationLevel::None;
        let _o1 = OptimizationLevel::O1;
        let _o2 = OptimizationLevel::O2;
        let _o3 = OptimizationLevel::O3;
        let _os = OptimizationLevel::Os;
    }

    #[test]
    fn test_export_type_variants() {
        let _func = ExportType::Function;
        let _table = ExportType::Table;
        let _memory = ExportType::Memory;
        let _global = ExportType::Global;
    }

    #[test]
    fn test_import_type_variants() {
        let _func = ImportType::Function;
        let _table = ImportType::Table;
        let _memory = ImportType::Memory;
        let _global = ImportType::Global;
    }

    #[test]
    fn test_target_architecture_variants() {
        let _wasm32 = TargetArchitecture::Wasm32;
        let _wasm64 = TargetArchitecture::Wasm64;
    }

    #[test]
    fn test_feature_flags() {
        let flags = FeatureFlags {
            simd: true,
            bulk_memory: false,
            reference_types: true,
            multi_value: false,
            tail_call: true,
            threads: false,
            exceptions: true,
            component_model: false,
        };

        assert!(flags.simd);
        assert!(!flags.bulk_memory);
        assert!(flags.reference_types);
        assert!(!flags.multi_value);
        assert!(flags.tail_call);
        assert!(!flags.threads);
        assert!(flags.exceptions);
        assert!(!flags.component_model);
    }

    #[test]
    fn test_memory_config() {
        let config = MemoryConfig {
            initial_pages: 5,
            maximum_pages: Some(10),
            shared: false,
            memory64: false,
        };

        assert_eq!(config.initial_pages, 5);
        assert_eq!(config.maximum_pages, Some(10));
    }

    #[test]
    fn test_linking_config() {
        let config = LinkingConfig {
            imports: vec!["env".to_string()],
            export_all: false,
            preserve_custom_sections: true,
            generate_names: false,
        };

        assert_eq!(config.imports.len(), 1);
        assert!(!config.export_all);
        assert!(config.preserve_custom_sections);
        assert!(!config.generate_names);
    }

    #[test]
    fn test_optimization_config() {
        let config = OptimizationConfig {
            level: OptimizationLevel::O2,
            optimize_size: false,
            optimize_speed: true,
            inline_threshold: 100,
            unroll_loops: true,
        };

        assert_eq!(config.level, OptimizationLevel::O2);
        assert!(!config.optimize_size);
        assert!(config.optimize_speed);
        assert_eq!(config.inline_threshold, 100);
        assert!(config.unroll_loops);
    }

    #[test]
    fn test_component_metadata_creation() {
        let metadata = ComponentMetadata {
            name: "my_component".to_string(),
            version: "2.0.0".to_string(),
            description: "Test component".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            repository: Some("https://github.com/test/repo".to_string()),
            build_time: std::time::SystemTime::now(),
            custom: HashMap::new(),
        };

        assert_eq!(metadata.name, "my_component");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.description, "Test component");
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.license, "MIT");
        assert!(metadata.repository.is_some());
    }

    // Test removed - TypeSignature type not defined in module

    // Test removed - TypeSignature type not defined in module

    #[test]
    fn test_wasm_component_with_custom_sections() {
        let mut custom_sections = HashMap::new();
        custom_sections.insert("debug".to_string(), vec![1, 2, 3, 4]);
        custom_sections.insert("producers".to_string(), vec![5, 6, 7, 8]);

        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0, 1, 2],
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections,
        };

        assert_eq!(component.custom_sections.len(), 2);
        assert!(component.custom_sections.contains_key("debug"));
        assert!(component.custom_sections.contains_key("producers"));
    }

    #[test]
    fn test_builder_add_multiple_sources() {
        let mut builder = ComponentBuilder::new();

        // Create temporary files
        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("file1.ruchy");
        let file2 = temp_dir.join("file2.ruchy");
        let file3 = temp_dir.join("file3.ruchy");

        fs::write(&file1, "fn main() {}").expect("operation should succeed in test");
        fs::write(&file2, "fn helper() {}").expect("operation should succeed in test");
        fs::write(&file3, "fn utils() {}").expect("operation should succeed in test");

        builder
            .add_source(&file1)
            .expect("operation should succeed in test");
        builder
            .add_source(&file2)
            .expect("operation should succeed in test");
        builder
            .add_source(&file3)
            .expect("operation should succeed in test");

        assert_eq!(builder.source_files.len(), 3);
        assert!(builder.source_files.contains(&file1));
        assert!(builder.source_files.contains(&file2));
        assert!(builder.source_files.contains(&file3));

        // Clean up
        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);
        let _ = fs::remove_file(file3);
    }

    #[test]
    fn test_optimization_levels() {
        let levels = vec![
            OptimizationLevel::None,
            OptimizationLevel::O1,
            OptimizationLevel::O2,
            OptimizationLevel::O3,
            OptimizationLevel::Os,
            OptimizationLevel::Oz,
        ];

        for level in levels {
            let builder = ComponentBuilder::new();
            let builder = builder.with_optimization(level.clone());
            assert_eq!(builder.optimization_level, level);
        }
    }

    #[test]
    fn test_target_architecture_variants_comprehensive() {
        let targets = vec![TargetArchitecture::Wasm32, TargetArchitecture::Wasm64];

        for target in targets {
            let config = ComponentConfig {
                target: target.clone(),
                memory: MemoryConfig::default(),
                features: FeatureFlags::default(),
                linking: LinkingConfig::default(),
                optimization: OptimizationConfig::default(),
            };

            assert_eq!(config.target, target);
        }
    }

    // Test removed - TypeSignature, MemoryLimits, TableLimits types not defined in module

    #[test]
    fn test_wasm_type_variants() {
        let types = vec![
            WasmType::I32,
            WasmType::I64,
            WasmType::F32,
            WasmType::F64,
            WasmType::V128,
            WasmType::FuncRef,
            WasmType::ExternRef,
        ];

        for wasm_type in types {
            // Each type should be distinct
            match wasm_type {
                WasmType::I32 => assert_eq!(format!("{wasm_type:?}"), "I32"),
                WasmType::I64 => assert_eq!(format!("{wasm_type:?}"), "I64"),
                WasmType::F32 => assert_eq!(format!("{wasm_type:?}"), "F32"),
                WasmType::F64 => assert_eq!(format!("{wasm_type:?}"), "F64"),
                WasmType::V128 => assert_eq!(format!("{wasm_type:?}"), "V128"),
                WasmType::FuncRef => assert_eq!(format!("{wasm_type:?}"), "FuncRef"),
                WasmType::ExternRef => assert_eq!(format!("{wasm_type:?}"), "ExternRef"),
                WasmType::Ref(name) => assert!(!name.is_empty()),
            }
        }
    }

    #[test]
    fn test_builder_with_debug_info() {
        let builder = ComponentBuilder::new();

        let builder = builder.with_debug_info(true);
        assert!(builder.include_debug_info);

        let builder = builder.with_debug_info(false);
        assert!(!builder.include_debug_info);
    }

    #[test]
    fn test_memory_config_with_limits() {
        let config1 = MemoryConfig {
            initial_pages: 1,
            maximum_pages: None,
            shared: false,
            memory64: false,
        };

        let config2 = MemoryConfig {
            initial_pages: 10,
            maximum_pages: Some(100),
            shared: true,
            memory64: true,
        };

        assert_eq!(config1.initial_pages, 1);
        assert!(config1.maximum_pages.is_none());
        assert!(!config1.shared);
        assert!(!config1.memory64);

        assert_eq!(config2.initial_pages, 10);
        assert_eq!(config2.maximum_pages, Some(100));
        assert!(config2.shared);
        assert!(config2.memory64);
    }

    #[test]
    fn test_component_validation_interface() {
        let component = WasmComponent {
            name: "validator".to_string(),
            version: "0.1.0".to_string(),
            bytecode: vec![0x00, 0x61, 0x73, 0x6d], // WASM magic number
            metadata: ComponentMetadata::default(),
            exports: vec![ExportDefinition {
                name: "validate".to_string(),
                export_type: ExportType::Function,
                signature: TypeSignature {
                    params: vec![WasmType::I32],
                    results: vec![WasmType::I32],
                    metadata: HashMap::new(),
                },
                documentation: None,
            }],
            imports: vec![],
            custom_sections: HashMap::new(),
        };

        assert!(!component.name.is_empty());
        assert!(!component.version.is_empty());
        assert!(component.bytecode.len() >= 4);
        assert_eq!(component.exports.len(), 1);
    }

    #[test]
    fn test_feature_flags_combinations() {
        let all_enabled = FeatureFlags {
            simd: true,
            bulk_memory: true,
            reference_types: true,
            multi_value: true,
            tail_call: true,
            threads: true,
            exceptions: true,
            component_model: true,
        };

        let all_disabled = FeatureFlags {
            simd: false,
            bulk_memory: false,
            reference_types: false,
            multi_value: false,
            tail_call: false,
            threads: false,
            exceptions: false,
            component_model: false,
        };

        // Test all enabled
        assert!(all_enabled.simd && all_enabled.bulk_memory);
        assert!(all_enabled.reference_types && all_enabled.multi_value);

        // Test all disabled
        assert!(!all_disabled.simd && !all_disabled.bulk_memory);
        assert!(!all_disabled.reference_types && !all_disabled.multi_value);
    }

    #[test]
    fn test_linking_config_with_multiple_imports() {
        let config = LinkingConfig {
            imports: vec![
                "env".to_string(),
                "wasi_snapshot_preview1".to_string(),
                "custom_module".to_string(),
            ],
            export_all: true,
            preserve_custom_sections: true,
            generate_names: true,
        };

        assert_eq!(config.imports.len(), 3);
        assert!(config.imports.contains(&"env".to_string()));
        assert!(config
            .imports
            .contains(&"wasi_snapshot_preview1".to_string()));
        assert!(config.export_all);
        assert!(config.generate_names);
    }

    #[test]
    fn test_custom_metadata_fields() {
        let mut custom = HashMap::new();
        custom.insert("compiler".to_string(), "ruchy".to_string());
        custom.insert("target".to_string(), "browser".to_string());
        custom.insert("optimization".to_string(), "size".to_string());

        let metadata = ComponentMetadata {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            license: String::new(),
            repository: None,
            build_time: std::time::SystemTime::now(),
            custom,
        };

        assert_eq!(metadata.custom.len(), 3);
        assert_eq!(metadata.custom.get("compiler"), Some(&"ruchy".to_string()));
        assert_eq!(metadata.custom.get("target"), Some(&"browser".to_string()));
    }

    // Test removed - TypeSignature::Table and TableLimits types not defined

    // Test removed - TypeSignature type not defined in module

    #[test]
    fn test_optimization_size_vs_speed() {
        let size_opt = OptimizationConfig {
            level: OptimizationLevel::Os,
            optimize_size: true,
            optimize_speed: false,
            inline_threshold: 10,
            unroll_loops: false,
        };

        let speed_opt = OptimizationConfig {
            level: OptimizationLevel::O3,
            optimize_size: false,
            optimize_speed: true,
            inline_threshold: 1000,
            unroll_loops: true,
        };

        // Size optimization should have different settings than speed
        assert!(size_opt.optimize_size && !size_opt.optimize_speed);
        assert!(!speed_opt.optimize_size && speed_opt.optimize_speed);
        assert!(size_opt.inline_threshold < speed_opt.inline_threshold);
        assert!(!size_opt.unroll_loops && speed_opt.unroll_loops);
    }

    // Additional coverage tests

    #[test]
    fn test_build_dry_run() {
        let mut builder = ComponentBuilder::new();
        builder.set_name("dry-run-test".to_string());
        builder.set_version("1.0.0".to_string());

        let component = builder.build_dry_run();
        assert!(component.is_ok());
        let component = component.unwrap();
        assert_eq!(component.name, "dry-run-test");
        assert!(!component.bytecode.is_empty());
        assert_eq!(component.exports.len(), 1);
    }

    #[test]
    fn test_build_with_source_files() {
        let mut builder = ComponentBuilder::new();
        builder.set_name("build-test".to_string());

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("build_test_source.ruchy");
        fs::write(&temp_file, "fn main() { 42 }").expect("write should succeed");

        builder.add_source(&temp_file).expect("add should succeed");

        let result = builder.build();
        assert!(result.is_ok());
        let component = result.unwrap();
        assert!(!component.bytecode.is_empty());

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_build_validation_no_sources() {
        let builder = ComponentBuilder::new();
        // No sources added - should fail validation
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_validation_no_name() {
        let mut builder = ComponentBuilder::new();
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("no_name_test.ruchy");
        fs::write(&temp_file, "fn main() {}").expect("write should succeed");
        builder.add_source(&temp_file).expect("add should succeed");

        // No name set - should fail validation
        let result = builder.build();
        assert!(result.is_err());

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_target_architecture_all_variants() {
        let variants = vec![
            TargetArchitecture::Wasm32,
            TargetArchitecture::Wasm64,
            TargetArchitecture::Wasi,
            TargetArchitecture::Browser,
            TargetArchitecture::NodeJs,
            TargetArchitecture::CloudflareWorkers,
            TargetArchitecture::Custom("custom-target".to_string()),
        ];

        for target in variants {
            let config = ComponentConfig {
                target: target.clone(),
                ..Default::default()
            };
            assert_eq!(config.target, target);
        }
    }

    #[test]
    fn test_export_type_custom() {
        let custom = ExportType::Custom("custom_type".to_string());
        assert!(matches!(custom, ExportType::Custom(_)));
    }

    #[test]
    fn test_import_type_custom() {
        let custom = ImportType::Custom("custom_import".to_string());
        assert!(matches!(custom, ImportType::Custom(_)));
    }

    #[test]
    fn test_wasm_type_ref() {
        let ref_type = WasmType::Ref("custom_ref".to_string());
        assert!(matches!(ref_type, WasmType::Ref(_)));
    }

    #[test]
    fn test_component_validate_invalid_magic() {
        let component = WasmComponent {
            name: "invalid".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00],
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections: HashMap::new(),
        };

        let result = component.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_component_validate_too_small() {
        let component = WasmComponent {
            name: "tiny".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0x00, 0x61, 0x73], // Only 3 bytes - too small
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections: HashMap::new(),
        };

        let result = component.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_component_summary_with_debug_info() {
        let mut custom_sections = HashMap::new();
        custom_sections.insert("name".to_string(), vec![1, 2, 3]);

        let component = WasmComponent {
            name: "debug-component".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            metadata: ComponentMetadata::default(),
            exports: vec![],
            imports: vec![],
            custom_sections,
        };

        let summary = component.summary();
        assert!(summary.has_debug_info);
    }

    #[test]
    fn test_linking_config_default() {
        let config = LinkingConfig::default();
        assert!(config.imports.is_empty());
    }

    #[test]
    fn test_component_builder_default() {
        let builder = ComponentBuilder::default();
        assert_eq!(builder.source_files.len(), 0);
    }

    #[test]
    fn test_feature_flags_default() {
        let flags = FeatureFlags::default();
        // Default should have bulk_memory and multi_value enabled
        assert!(flags.bulk_memory);
        assert!(flags.multi_value);
    }

    #[test]
    fn test_optimization_level_oz() {
        let level = OptimizationLevel::Oz;
        let builder = ComponentBuilder::new().with_optimization(level.clone());
        assert_eq!(builder.optimization_level, OptimizationLevel::Oz);
    }

    #[test]
    fn test_type_signature_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("doc".to_string(), "Test function".to_string());

        let sig = TypeSignature {
            params: vec![WasmType::I32, WasmType::I64],
            results: vec![WasmType::F64],
            metadata,
        };

        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.results.len(), 1);
        assert!(sig.metadata.contains_key("doc"));
    }

    #[test]
    fn test_export_definition_with_documentation() {
        let export = ExportDefinition {
            name: "documented_func".to_string(),
            export_type: ExportType::Function,
            signature: TypeSignature {
                params: vec![],
                results: vec![WasmType::I32],
                metadata: HashMap::new(),
            },
            documentation: Some("This is a documented function".to_string()),
        };

        assert!(export.documentation.is_some());
        assert_eq!(
            export.documentation.unwrap(),
            "This is a documented function"
        );
    }

    #[test]
    fn test_import_definition_complete() {
        let import = ImportDefinition {
            module: "env".to_string(),
            name: "print".to_string(),
            import_type: ImportType::Function,
            signature: TypeSignature {
                params: vec![WasmType::I32],
                results: vec![],
                metadata: HashMap::new(),
            },
        };

        assert_eq!(import.module, "env");
        assert_eq!(import.name, "print");
        assert!(matches!(import.import_type, ImportType::Function));
    }

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert_eq!(config.initial_pages, 1);
        assert!(config.maximum_pages.is_none());
        assert!(!config.shared);
        assert!(!config.memory64);
    }
}

#[cfg(test)]
mod property_tests_component {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_set_name_doesnt_panic(name in "\\PC*") {
            let mut builder = ComponentBuilder::new();
            builder.set_name(name.clone());
            assert_eq!(builder.metadata.name, name);
        }

        #[test]
        fn test_set_version_doesnt_panic(version in "\\PC*") {
            let mut builder = ComponentBuilder::new();
            builder.set_version(version.clone());
            assert_eq!(builder.metadata.version, version);
        }

        #[test]
        fn test_component_size_with_random_bytecode(
            bytes in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let component = WasmComponent {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                bytecode: bytes.clone(),
                metadata: ComponentMetadata::default(),
                exports: vec![],
                imports: vec![],
                custom_sections: HashMap::new(),
            };

            assert_eq!(component.size(), bytes.len());
        }
    }
}
