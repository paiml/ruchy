//! WIT (WebAssembly Interface Type) generation for Ruchy (RUCHY-0819)
//!
//! Generates WIT interface definitions from Ruchy code for component interoperability.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::Path;
/// WIT interface definition
#[derive(Debug, Clone)]
pub struct WitInterface {
    /// Interface name
    pub name: String,
    /// Interface version
    pub version: String,
    /// Package information
    pub package: PackageInfo,
    /// Type definitions
    pub types: Vec<TypeDefinition>,
    /// Function definitions
    pub functions: Vec<FunctionDefinition>,
    /// Resource definitions
    pub resources: Vec<ResourceDefinition>,
    /// World definition
    pub world: Option<WorldDefinition>,
}
/// WIT interface generator
pub struct WitGenerator {
    /// Generation configuration
    config: WitConfig,
    /// Type registry
    _type_registry: TypeRegistry,
    /// Import tracking
    _imports: HashSet<String>,
}
/// Configuration for WIT generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitConfig {
    /// Generate documentation comments
    pub include_docs: bool,
    /// Generate type aliases
    pub use_type_aliases: bool,
    /// Generate resource types
    pub generate_resources: bool,
    /// Component model version
    pub component_model_version: String,
    /// Custom type mappings
    pub type_mappings: HashMap<String, String>,
    /// World name
    pub world_name: Option<String>,
}
/// Interface definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    /// Interface name
    pub name: String,
    /// Interface functions
    pub functions: Vec<InterfaceFunction>,
    /// Interface types
    pub types: Vec<InterfaceType>,
    /// Documentation
    pub documentation: Option<String>,
}
/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package namespace
    pub namespace: String,
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
}
/// Type definition in WIT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// Type name
    pub name: String,
    /// Type kind
    pub kind: TypeKind,
    /// Documentation
    pub documentation: Option<String>,
}
/// Function definition in WIT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Parameters
    pub params: Vec<Parameter>,
    /// Return type
    pub return_type: Option<WitType>,
    /// Whether function is async
    pub is_async: bool,
    /// Documentation
    pub documentation: Option<String>,
}
/// Resource definition in WIT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    /// Resource name
    pub name: String,
    /// Resource methods
    pub methods: Vec<ResourceMethod>,
    /// Constructor
    pub constructor: Option<FunctionDefinition>,
    /// Documentation
    pub documentation: Option<String>,
}
/// World definition for component composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDefinition {
    /// World name
    pub name: String,
    /// Imports
    pub imports: Vec<WorldImport>,
    /// Exports
    pub exports: Vec<WorldExport>,
    /// Documentation
    pub documentation: Option<String>,
}
/// Type kinds in WIT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    /// Record type (struct)
    Record(Vec<Field>),
    /// Variant type (enum)
    Variant(Vec<VariantCase>),
    /// Flags type (bitflags)
    Flags(Vec<String>),
    /// Tuple type
    Tuple(Vec<WitType>),
    /// List type
    List(Box<WitType>),
    /// Option type
    Option(Box<WitType>),
    /// Result type
    Result {
        ok: Option<Box<WitType>>,
        err: Option<Box<WitType>>,
    },
    /// Type alias
    Alias(Box<WitType>),
}
/// WIT type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WitType {
    /// Boolean
    Bool,
    /// Unsigned 8-bit integer
    U8,
    /// Unsigned 16-bit integer
    U16,
    /// Unsigned 32-bit integer
    U32,
    /// Unsigned 64-bit integer
    U64,
    /// Signed 8-bit integer
    S8,
    /// Signed 16-bit integer
    S16,
    /// Signed 32-bit integer
    S32,
    /// Signed 64-bit integer
    S64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Character
    Char,
    /// String
    String,
    /// Named type reference
    Named(String),
    /// List type
    List(Box<WitType>),
    /// Option type
    Option(Box<WitType>),
    /// Result type
    Result {
        ok: Option<Box<WitType>>,
        err: Option<Box<WitType>>,
    },
    /// Tuple type
    Tuple(Vec<WitType>),
}
/// Record field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: WitType,
    /// Documentation
    pub documentation: Option<String>,
}
/// Variant case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantCase {
    /// Case name
    pub name: String,
    /// Associated data
    pub payload: Option<WitType>,
    /// Documentation
    pub documentation: Option<String>,
}
/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: WitType,
}
/// Resource method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMethod {
    /// Method name
    pub name: String,
    /// Method kind
    pub kind: MethodKind,
    /// Method definition
    pub function: FunctionDefinition,
}
/// Method kinds for resources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodKind {
    /// Constructor method
    Constructor,
    /// Static method
    Static,
    /// Instance method
    Instance,
}
/// Interface function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceFunction {
    /// Function name
    pub name: String,
    /// Parameters
    pub params: Vec<(String, String)>,
    /// Return type
    pub return_type: Option<String>,
}
/// Interface type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceType {
    /// Type name
    pub name: String,
    /// Type definition
    pub definition: String,
}
/// World import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldImport {
    /// Import name
    pub name: String,
    /// Import interface
    pub interface: String,
}
/// World export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldExport {
    /// Export name
    pub name: String,
    /// Export interface
    pub interface: String,
}
/// Type registry for managing type definitions
struct TypeRegistry {
    /// Registered types
    _types: HashMap<String, TypeDefinition>,
    /// Type dependencies
    _dependencies: HashMap<String, HashSet<String>>,
}
impl Default for WitConfig {
    fn default() -> Self {
        Self {
            include_docs: true,
            use_type_aliases: true,
            generate_resources: true,
            component_model_version: "0.2.0".to_string(),
            type_mappings: HashMap::new(),
            world_name: None,
        }
    }
}
impl Default for WitGenerator {
    fn default() -> Self {
        Self::new()
    }
}
impl WitGenerator {
    /// Create a new WIT generator with default config
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::wit::WitGenerator;
    ///
    /// let instance = WitGenerator::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            config: WitConfig::default(),
            _type_registry: TypeRegistry::new(),
            _imports: HashSet::new(),
        }
    }
    /// Create a new WIT generator with specific config
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::wit::WitGenerator;
    ///
    /// let mut instance = WitGenerator::new();
    /// let result = instance.new_with_config();
    /// // Verify behavior
    /// ```
    pub fn new_with_config(config: WitConfig) -> Self {
        Self {
            config,
            _type_registry: TypeRegistry::new(),
            _imports: HashSet::new(),
        }
    }
    /// Set the world name
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::wit::with_world;
    ///
    /// let result = with_world("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn with_world(&mut self, world: &str) -> &mut Self {
        self.config.world_name = Some(world.to_string());
        self
    }
    /// Generate WIT interface from component
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::wit::WitGenerator;
    ///
    /// let mut instance = WitGenerator::new();
    /// let result = instance.generate();
    /// // Verify behavior
    /// ```
    pub fn generate(
        &mut self,
        component: &super::component::WasmComponent,
    ) -> Result<WitInterface> {
        self.generate_from_component(component)
    }
    /// Generate WIT interface from component
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::wit::generate_from_component;
    ///
    /// let result = generate_from_component(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_from_component(
        &mut self,
        _component: &super::component::WasmComponent,
    ) -> Result<WitInterface> {
        // In a real implementation, analyze the component's exports and imports
        self.generate_default()
    }
    /// Generate WIT interface from Ruchy source
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::wit::generate_from_source;
    ///
    /// let result = generate_from_source("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_from_source(&mut self, _source: &str) -> Result<WitInterface> {
        // In a real implementation, this would:
        // 1. Parse Ruchy source code
        // 2. Extract type definitions
        // 3. Extract function signatures
        // 4. Generate corresponding WIT definitions
        // For now, create a sample interface
        let interface = WitInterface {
            name: "ruchy-component".to_string(),
            version: "0.1.0".to_string(),
            package: PackageInfo {
                namespace: "ruchy".to_string(),
                name: "component".to_string(),
                version: "0.1.0".to_string(),
            },
            types: vec![TypeDefinition {
                name: "request".to_string(),
                kind: TypeKind::Record(vec![
                    Field {
                        name: "method".to_string(),
                        field_type: WitType::String,
                        documentation: Some("HTTP method".to_string()),
                    },
                    Field {
                        name: "path".to_string(),
                        field_type: WitType::String,
                        documentation: Some("Request path".to_string()),
                    },
                ]),
                documentation: Some("HTTP request type".to_string()),
            }],
            functions: vec![FunctionDefinition {
                name: "handle-request".to_string(),
                params: vec![Parameter {
                    name: "req".to_string(),
                    param_type: WitType::Named("request".to_string()),
                }],
                return_type: Some(WitType::String),
                is_async: false,
                documentation: Some("Handle an HTTP request".to_string()),
            }],
            resources: vec![],
            world: Some(WorldDefinition {
                name: "http-handler".to_string(),
                imports: vec![],
                exports: vec![WorldExport {
                    name: "handler".to_string(),
                    interface: "ruchy:component/handler".to_string(),
                }],
                documentation: Some("HTTP handler world".to_string()),
            }),
        };
        Ok(interface)
    }
    /// Generate a default WIT interface
    fn generate_default(&mut self) -> Result<WitInterface> {
        self.generate_from_source("")
    }
    /// Add a custom type mapping
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::wit::WitGenerator;
    ///
    /// let mut instance = WitGenerator::new();
    /// let result = instance.add_type_mapping();
    /// // Verify behavior
    /// ```
    pub fn add_type_mapping(&mut self, ruchy_type: String, wit_type: String) {
        self.config.type_mappings.insert(ruchy_type, wit_type);
    }
    /// Generate WIT file content
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::wit::generate_wit_file;
    ///
    /// let result = generate_wit_file(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_wit_file(&self, interface: &WitInterface) -> String {
        let mut wit = String::new();
        // Package declaration
        wit.push_str(&format!(
            "package {}:{}/{}@{};\n\n",
            interface.package.namespace, interface.package.name, interface.name, interface.version
        ));
        // Interface declaration
        wit.push_str(&format!("interface {} {{\n", interface.name));
        // Type definitions
        for type_def in &interface.types {
            if let Some(doc) = &type_def.documentation {
                wit.push_str(&format!("  /// {doc}\n"));
            }
            wit.push_str(&format!("  {}\n\n", self.format_type_definition(type_def)));
        }
        // Function definitions
        for func in &interface.functions {
            if let Some(doc) = &func.documentation {
                wit.push_str(&format!("  /// {doc}\n"));
            }
            wit.push_str(&format!("  {}\n", self.format_function(func)));
        }
        wit.push_str("}\n\n");
        // World definition
        if let Some(world) = &interface.world {
            wit.push_str(&self.format_world(world));
        }
        wit
    }
    fn format_type_definition(&self, type_def: &TypeDefinition) -> String {
        match &type_def.kind {
            TypeKind::Record(fields) => {
                let mut s = format!("record {} {{\n", type_def.name);
                for field in fields {
                    if let Some(doc) = &field.documentation {
                        s.push_str(&format!("    /// {doc}\n"));
                    }
                    s.push_str(&format!(
                        "    {}: {},\n",
                        field.name,
                        self.format_wit_type(&field.field_type)
                    ));
                }
                s.push_str("  }");
                s
            }
            TypeKind::Variant(cases) => {
                let mut s = format!("variant {} {{\n", type_def.name);
                for case in cases {
                    if let Some(doc) = &case.documentation {
                        s.push_str(&format!("    /// {doc}\n"));
                    }
                    if let Some(payload) = &case.payload {
                        s.push_str(&format!(
                            "    {}({}),\n",
                            case.name,
                            self.format_wit_type(payload)
                        ));
                    } else {
                        s.push_str(&format!("    {},\n", case.name));
                    }
                }
                s.push_str("  }");
                s
            }
            TypeKind::Flags(flags) => {
                let mut s = format!("flags {} {{\n", type_def.name);
                for flag in flags {
                    s.push_str(&format!("    {flag},\n"));
                }
                s.push_str("  }");
                s
            }
            TypeKind::Alias(wit_type) => {
                format!(
                    "type {} = {}",
                    type_def.name,
                    self.format_wit_type(wit_type)
                )
            }
            _ => format!("type {}", type_def.name),
        }
    }
    fn format_function(&self, func: &FunctionDefinition) -> String {
        let params = func
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.format_wit_type(&p.param_type)))
            .collect::<Vec<_>>()
            .join(", ");
        let return_part = if let Some(ret) = &func.return_type {
            format!(" -> {}", self.format_wit_type(ret))
        } else {
            String::new()
        };
        format!("{}: func({}){};", func.name, params, return_part)
    }
    fn format_wit_type(&self, wit_type: &WitType) -> String {
        match wit_type {
            WitType::Bool => "bool".to_string(),
            WitType::U8 => "u8".to_string(),
            WitType::U16 => "u16".to_string(),
            WitType::U32 => "u32".to_string(),
            WitType::U64 => "u64".to_string(),
            WitType::S8 => "s8".to_string(),
            WitType::S16 => "s16".to_string(),
            WitType::S32 => "s32".to_string(),
            WitType::S64 => "s64".to_string(),
            WitType::F32 => "f32".to_string(),
            WitType::F64 => "f64".to_string(),
            WitType::Char => "char".to_string(),
            WitType::String => "string".to_string(),
            WitType::Named(name) => name.clone(),
            WitType::List(inner) => format!("list<{}>", self.format_wit_type(inner)),
            WitType::Option(inner) => format!("option<{}>", self.format_wit_type(inner)),
            WitType::Result { ok, err } => {
                let ok_str = ok
                    .as_ref()
                    .map_or_else(|| "_".to_string(), |t| self.format_wit_type(t));
                let err_str = err
                    .as_ref()
                    .map_or_else(|| "_".to_string(), |t| self.format_wit_type(t));
                format!("result<{ok_str}, {err_str}>")
            }
            WitType::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(|t| self.format_wit_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple<{types_str}>")
            }
        }
    }
    fn format_world(&self, world: &WorldDefinition) -> String {
        let mut s = String::new();
        if let Some(doc) = &world.documentation {
            s.push_str(&format!("/// {doc}\n"));
        }
        s.push_str(&format!("world {} {{\n", world.name));
        // Imports
        for import in &world.imports {
            s.push_str(&format!(
                "  import {}: {};\n",
                import.name, import.interface
            ));
        }
        // Exports
        for export in &world.exports {
            s.push_str(&format!(
                "  export {}: {};\n",
                export.name, export.interface
            ));
        }
        s.push_str("}\n");
        s
    }
}
impl fmt::Display for WitInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let generator = WitGenerator::new();
        write!(f, "{}", generator.generate_wit_file(self))
    }
}
impl WitInterface {
    /// Save the WIT interface to a file
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::wit::save;
    ///
    /// let result = save(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let generator = WitGenerator::new();
        let wit_content = generator.generate_wit_file(self);
        fs::write(path.as_ref(), wit_content)
            .with_context(|| format!("Failed to write WIT file to {}", path.as_ref().display()))?;
        Ok(())
    }
}
impl TypeRegistry {
    fn new() -> Self {
        Self {
            _types: HashMap::new(),
            _dependencies: HashMap::new(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wit_config_default() {
        let config = WitConfig::default();
        assert!(config.include_docs);
        assert!(config.use_type_aliases);
        assert!(config.generate_resources);
        assert_eq!(config.component_model_version, "0.2.0");
        assert!(config.type_mappings.is_empty());
        assert_eq!(config.world_name, None);
    }

    #[test]
    fn test_wit_config_custom() {
        let mut mappings = HashMap::new();
        mappings.insert("String".to_string(), "string".to_string());

        let config = WitConfig {
            include_docs: false,
            use_type_aliases: false,
            generate_resources: false,
            component_model_version: "0.3.0".to_string(),
            type_mappings: mappings.clone(),
            world_name: Some("test-world".to_string()),
        };

        assert!(!config.include_docs);
        assert!(!config.use_type_aliases);
        assert!(!config.generate_resources);
        assert_eq!(config.component_model_version, "0.3.0");
        assert_eq!(config.type_mappings, mappings);
        assert_eq!(config.world_name, Some("test-world".to_string()));
    }

    #[test]
    fn test_wit_generator_new() {
        let generator = WitGenerator::new();
        assert!(generator.config.include_docs);
        assert!(generator.config.use_type_aliases);
        assert!(generator.config.generate_resources);
    }

    #[test]
    fn test_wit_generator_new_with_config() {
        let config = WitConfig {
            include_docs: false,
            use_type_aliases: false,
            generate_resources: false,
            component_model_version: "0.1.0".to_string(),
            type_mappings: HashMap::new(),
            world_name: Some("custom".to_string()),
        };

        let generator = WitGenerator::new_with_config(config);
        assert!(!generator.config.include_docs);
        assert_eq!(generator.config.world_name, Some("custom".to_string()));
    }

    #[test]
    fn test_wit_generator_with_world() {
        let mut generator = WitGenerator::new();
        generator.with_world("my-world");
        assert_eq!(generator.config.world_name, Some("my-world".to_string()));
    }

    #[test]
    fn test_wit_generator_add_type_mapping() {
        let mut generator = WitGenerator::new();
        generator.add_type_mapping("Vec<T>".to_string(), "list<T>".to_string());
        assert_eq!(
            generator.config.type_mappings.get("Vec<T>"),
            Some(&"list<T>".to_string())
        );
    }

    #[test]
    fn test_package_info_creation() {
        let package = PackageInfo {
            namespace: "test".to_string(),
            name: "my-package".to_string(),
            version: "1.0.0".to_string(),
        };

        assert_eq!(package.namespace, "test");
        assert_eq!(package.name, "my-package");
        assert_eq!(package.version, "1.0.0");
    }

    #[test]
    fn test_type_definition_creation() {
        let type_def = TypeDefinition {
            name: "MyType".to_string(),
            kind: TypeKind::Record(vec![]),
            documentation: Some("Test type".to_string()),
        };

        assert_eq!(type_def.name, "MyType");
        assert!(matches!(type_def.kind, TypeKind::Record(_)));
        assert_eq!(type_def.documentation, Some("Test type".to_string()));
    }

    #[test]
    fn test_function_definition_creation() {
        let func = FunctionDefinition {
            name: "test_func".to_string(),
            params: vec![],
            return_type: None,
            is_async: false,
            documentation: Some("Test function".to_string()),
        };

        assert_eq!(func.name, "test_func");
        assert!(func.params.is_empty());
        assert!(func.return_type.is_none());
        assert!(!func.is_async);
    }

    #[test]
    fn test_wit_interface_creation() {
        let interface = WitInterface {
            name: "test-interface".to_string(),
            version: "1.0.0".to_string(),
            package: PackageInfo {
                namespace: "test".to_string(),
                name: "package".to_string(),
                version: "1.0.0".to_string(),
            },
            types: vec![],
            functions: vec![],
            resources: vec![],
            world: None,
        };

        assert_eq!(interface.name, "test-interface");
        assert_eq!(interface.version, "1.0.0");
        assert!(interface.types.is_empty());
        assert!(interface.functions.is_empty());
        assert!(interface.resources.is_empty());
        assert!(interface.world.is_none());
    }

    #[test]
    fn test_wit_type_variants() {
        // Test that WitType variants exist and can be created
        let _bool_type = WitType::Bool;
        let _u32_type = WitType::U32;
        let _string_type = WitType::String;

        // Test pattern matching works
        let wit_type = WitType::Bool;
        match wit_type {
            WitType::Bool => {}
            _ => assert!(false, "Expected Bool variant"),
        }
    }

    #[test]
    fn test_wit_interface_save() {
        let interface = WitInterface {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            package: PackageInfo {
                namespace: "test".to_string(),
                name: "test".to_string(),
                version: "1.0.0".to_string(),
            },
            types: vec![],
            functions: vec![],
            resources: vec![],
            world: None,
        };

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_wit_interface.wit");

        // Should create the file
        let result = interface.save(&temp_file);
        assert!(result.is_ok());

        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_generate_wit_file() {
        let generator = WitGenerator::new();
        let interface = WitInterface {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            package: PackageInfo {
                namespace: "example".to_string(),
                name: "test".to_string(),
                version: "1.0.0".to_string(),
            },
            types: vec![],
            functions: vec![],
            resources: vec![],
            world: Some(WorldDefinition {
                name: "test-world".to_string(),
                imports: vec![],
                exports: vec![],
                documentation: None,
            }),
        };

        let wit_content = generator.generate_wit_file(&interface);
        assert!(wit_content.contains("package example:test/test@1.0.0"));
        assert!(wit_content.contains("world test-world"));
    }

    #[test]
    fn test_interface_function_creation() {
        let func = InterfaceFunction {
            name: "my_func".to_string(),
            params: vec![],
            return_type: None,
        };

        assert_eq!(func.name, "my_func");
        assert!(func.params.is_empty());
        assert!(func.return_type.is_none());
    }

    #[test]
    fn test_interface_type_creation() {
        let ty = InterfaceType {
            name: "MyType".to_string(),
            definition: "record my-type { }".to_string(),
        };

        assert_eq!(ty.name, "MyType");
        assert_eq!(ty.definition, "record my-type { }");
    }

    #[test]
    fn test_resource_definition_creation() {
        let resource = ResourceDefinition {
            name: "my-resource".to_string(),
            methods: vec![],
            constructor: None,
            documentation: Some("Resource doc".to_string()),
        };

        assert_eq!(resource.name, "my-resource");
        assert!(resource.methods.is_empty());
        assert!(resource.constructor.is_none());
        assert_eq!(resource.documentation, Some("Resource doc".to_string()));
    }

    #[test]
    fn test_world_definition_creation() {
        let world = WorldDefinition {
            name: "my-world".to_string(),
            imports: vec![],
            exports: vec![],
            documentation: None,
        };

        assert_eq!(world.name, "my-world");
        assert!(world.imports.is_empty());
        assert!(world.exports.is_empty());
        assert!(world.documentation.is_none());
    }

    #[test]
    fn test_parameter_creation() {
        let param = Parameter {
            name: "input".to_string(),
            param_type: WitType::String,
        };

        assert_eq!(param.name, "input");
        assert!(matches!(param.param_type, WitType::String));
    }

    #[test]
    fn test_type_registry_creation() {
        // Just test that we can create a TypeRegistry
        let registry = TypeRegistry::new();
        // Can't test internals since fields are private, but at least verify it compiles
        let _registry2 = registry; // Move to verify it's a valid type
    }
}

#[cfg(test)]
mod property_tests_wit {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_wit_generator_doesnt_panic_on_random_world_names(name in "\\PC*") {
            let mut generator = WitGenerator::new();
            generator.with_world(&name);
            assert_eq!(generator.config.world_name, Some(name));
        }

        #[test]
        fn test_add_type_mapping_doesnt_panic(
            key in "\\PC*",
            value in "\\PC*"
        ) {
            let mut generator = WitGenerator::new();
            generator.add_type_mapping(key.clone(), value.clone());
            assert_eq!(generator.config.type_mappings.get(&key), Some(&value));
        }

        #[test]
        fn test_package_info_with_random_strings(
            namespace in "\\PC*",
            name in "\\PC*",
            version in "\\PC*"
        ) {
            let package = PackageInfo {
                namespace: namespace.clone(),
                name: name.clone(),
                version: version.clone(),
            };

            assert_eq!(package.namespace, namespace);
            assert_eq!(package.name, name);
            assert_eq!(package.version, version);
        }
    }
}
