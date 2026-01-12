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
                        Self::format_wit_type(&field.field_type)
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
                            Self::format_wit_type(payload)
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
                    Self::format_wit_type(wit_type)
                )
            }
            _ => format!("type {}", type_def.name),
        }
    }
    fn format_function(&self, func: &FunctionDefinition) -> String {
        let params = func
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, Self::format_wit_type(&p.param_type)))
            .collect::<Vec<_>>()
            .join(", ");
        let return_part = if let Some(ret) = &func.return_type {
            format!(" -> {}", Self::format_wit_type(ret))
        } else {
            String::new()
        };
        format!("{}: func({}){};", func.name, params, return_part)
    }
    fn format_wit_type(wit_type: &WitType) -> String {
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
            WitType::List(inner) => format!("list<{}>", Self::format_wit_type(inner)),
            WitType::Option(inner) => format!("option<{}>", Self::format_wit_type(inner)),
            WitType::Result { ok, err } => {
                let ok_str = ok
                    .as_ref()
                    .map_or_else(|| "_".to_string(), |t| Self::format_wit_type(t));
                let err_str = err
                    .as_ref()
                    .map_or_else(|| "_".to_string(), |t| Self::format_wit_type(t));
                format!("result<{ok_str}, {err_str}>")
            }
            WitType::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(Self::format_wit_type)
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
            _ => panic!("Expected Bool variant"),
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
mod additional_tests {
    use super::*;

    // WitType format tests
    #[test]
    fn test_format_wit_type_bool() {
        let result = WitGenerator::format_wit_type(&WitType::Bool);
        assert_eq!(result, "bool");
    }

    #[test]
    fn test_format_wit_type_u8() {
        let result = WitGenerator::format_wit_type(&WitType::U8);
        assert_eq!(result, "u8");
    }

    #[test]
    fn test_format_wit_type_u16() {
        let result = WitGenerator::format_wit_type(&WitType::U16);
        assert_eq!(result, "u16");
    }

    #[test]
    fn test_format_wit_type_u32() {
        let result = WitGenerator::format_wit_type(&WitType::U32);
        assert_eq!(result, "u32");
    }

    #[test]
    fn test_format_wit_type_u64() {
        let result = WitGenerator::format_wit_type(&WitType::U64);
        assert_eq!(result, "u64");
    }

    #[test]
    fn test_format_wit_type_s8() {
        let result = WitGenerator::format_wit_type(&WitType::S8);
        assert_eq!(result, "s8");
    }

    #[test]
    fn test_format_wit_type_s16() {
        let result = WitGenerator::format_wit_type(&WitType::S16);
        assert_eq!(result, "s16");
    }

    #[test]
    fn test_format_wit_type_s32() {
        let result = WitGenerator::format_wit_type(&WitType::S32);
        assert_eq!(result, "s32");
    }

    #[test]
    fn test_format_wit_type_s64() {
        let result = WitGenerator::format_wit_type(&WitType::S64);
        assert_eq!(result, "s64");
    }

    #[test]
    fn test_format_wit_type_f32() {
        let result = WitGenerator::format_wit_type(&WitType::F32);
        assert_eq!(result, "f32");
    }

    #[test]
    fn test_format_wit_type_f64() {
        let result = WitGenerator::format_wit_type(&WitType::F64);
        assert_eq!(result, "f64");
    }

    #[test]
    fn test_format_wit_type_char() {
        let result = WitGenerator::format_wit_type(&WitType::Char);
        assert_eq!(result, "char");
    }

    #[test]
    fn test_format_wit_type_string() {
        let result = WitGenerator::format_wit_type(&WitType::String);
        assert_eq!(result, "string");
    }

    #[test]
    fn test_format_wit_type_named() {
        let result = WitGenerator::format_wit_type(&WitType::Named("my-type".to_string()));
        assert_eq!(result, "my-type");
    }

    #[test]
    fn test_format_wit_type_list() {
        let result = WitGenerator::format_wit_type(&WitType::List(Box::new(WitType::U32)));
        assert_eq!(result, "list<u32>");
    }

    #[test]
    fn test_format_wit_type_option() {
        let result = WitGenerator::format_wit_type(&WitType::Option(Box::new(WitType::String)));
        assert_eq!(result, "option<string>");
    }

    #[test]
    fn test_format_wit_type_result_both() {
        let result = WitGenerator::format_wit_type(&WitType::Result {
            ok: Some(Box::new(WitType::U32)),
            err: Some(Box::new(WitType::String)),
        });
        assert_eq!(result, "result<u32, string>");
    }

    #[test]
    fn test_format_wit_type_result_ok_only() {
        let result = WitGenerator::format_wit_type(&WitType::Result {
            ok: Some(Box::new(WitType::Bool)),
            err: None,
        });
        assert_eq!(result, "result<bool, _>");
    }

    #[test]
    fn test_format_wit_type_result_err_only() {
        let result = WitGenerator::format_wit_type(&WitType::Result {
            ok: None,
            err: Some(Box::new(WitType::String)),
        });
        assert_eq!(result, "result<_, string>");
    }

    #[test]
    fn test_format_wit_type_result_neither() {
        let result = WitGenerator::format_wit_type(&WitType::Result {
            ok: None,
            err: None,
        });
        assert_eq!(result, "result<_, _>");
    }

    #[test]
    fn test_format_wit_type_tuple() {
        let result = WitGenerator::format_wit_type(&WitType::Tuple(vec![
            WitType::U32,
            WitType::String,
            WitType::Bool,
        ]));
        assert_eq!(result, "tuple<u32, string, bool>");
    }

    #[test]
    fn test_format_wit_type_tuple_empty() {
        let result = WitGenerator::format_wit_type(&WitType::Tuple(vec![]));
        assert_eq!(result, "tuple<>");
    }

    #[test]
    fn test_format_wit_type_nested() {
        let result = WitGenerator::format_wit_type(&WitType::List(Box::new(WitType::Option(
            Box::new(WitType::Result {
                ok: Some(Box::new(WitType::U32)),
                err: Some(Box::new(WitType::String)),
            }),
        ))));
        assert_eq!(result, "list<option<result<u32, string>>>");
    }

    // TypeKind format tests
    #[test]
    fn test_format_type_definition_record() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "person".to_string(),
            kind: TypeKind::Record(vec![
                Field {
                    name: "name".to_string(),
                    field_type: WitType::String,
                    documentation: Some("Person's name".to_string()),
                },
                Field {
                    name: "age".to_string(),
                    field_type: WitType::U32,
                    documentation: None,
                },
            ]),
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert!(result.contains("record person"));
        assert!(result.contains("name: string"));
        assert!(result.contains("age: u32"));
        assert!(result.contains("/// Person's name"));
    }

    #[test]
    fn test_format_type_definition_variant() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "status".to_string(),
            kind: TypeKind::Variant(vec![
                VariantCase {
                    name: "ok".to_string(),
                    payload: Some(WitType::U32),
                    documentation: Some("Success case".to_string()),
                },
                VariantCase {
                    name: "error".to_string(),
                    payload: None,
                    documentation: None,
                },
            ]),
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert!(result.contains("variant status"));
        assert!(result.contains("ok(u32)"));
        assert!(result.contains("error,"));
        assert!(result.contains("/// Success case"));
    }

    #[test]
    fn test_format_type_definition_flags() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "permissions".to_string(),
            kind: TypeKind::Flags(vec![
                "read".to_string(),
                "write".to_string(),
                "execute".to_string(),
            ]),
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert!(result.contains("flags permissions"));
        assert!(result.contains("read,"));
        assert!(result.contains("write,"));
        assert!(result.contains("execute,"));
    }

    #[test]
    fn test_format_type_definition_alias() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "my-string".to_string(),
            kind: TypeKind::Alias(Box::new(WitType::String)),
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert_eq!(result, "type my-string = string");
    }

    #[test]
    fn test_format_type_definition_tuple() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "pair".to_string(),
            kind: TypeKind::Tuple(vec![WitType::U32, WitType::String]),
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert!(result.contains("type pair"));
    }

    #[test]
    fn test_format_type_definition_list() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "numbers".to_string(),
            kind: TypeKind::List(Box::new(WitType::U32)),
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert!(result.contains("type numbers"));
    }

    #[test]
    fn test_format_type_definition_option() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "maybe-string".to_string(),
            kind: TypeKind::Option(Box::new(WitType::String)),
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert!(result.contains("type maybe-string"));
    }

    #[test]
    fn test_format_type_definition_result() {
        let generator = WitGenerator::new();
        let type_def = TypeDefinition {
            name: "my-result".to_string(),
            kind: TypeKind::Result {
                ok: Some(Box::new(WitType::U32)),
                err: Some(Box::new(WitType::String)),
            },
            documentation: None,
        };
        let result = generator.format_type_definition(&type_def);
        assert!(result.contains("type my-result"));
    }

    // Function format tests
    #[test]
    fn test_format_function_with_params_and_return() {
        let generator = WitGenerator::new();
        let func = FunctionDefinition {
            name: "greet".to_string(),
            params: vec![
                Parameter {
                    name: "name".to_string(),
                    param_type: WitType::String,
                },
                Parameter {
                    name: "count".to_string(),
                    param_type: WitType::U32,
                },
            ],
            return_type: Some(WitType::String),
            is_async: false,
            documentation: None,
        };
        let result = generator.format_function(&func);
        assert_eq!(result, "greet: func(name: string, count: u32) -> string;");
    }

    #[test]
    fn test_format_function_no_params() {
        let generator = WitGenerator::new();
        let func = FunctionDefinition {
            name: "get-time".to_string(),
            params: vec![],
            return_type: Some(WitType::U64),
            is_async: false,
            documentation: None,
        };
        let result = generator.format_function(&func);
        assert_eq!(result, "get-time: func() -> u64;");
    }

    #[test]
    fn test_format_function_no_return() {
        let generator = WitGenerator::new();
        let func = FunctionDefinition {
            name: "log".to_string(),
            params: vec![Parameter {
                name: "msg".to_string(),
                param_type: WitType::String,
            }],
            return_type: None,
            is_async: false,
            documentation: None,
        };
        let result = generator.format_function(&func);
        assert_eq!(result, "log: func(msg: string);");
    }

    // World format tests
    #[test]
    fn test_format_world_with_imports_and_exports() {
        let generator = WitGenerator::new();
        let world = WorldDefinition {
            name: "my-world".to_string(),
            imports: vec![
                WorldImport {
                    name: "console".to_string(),
                    interface: "wasi:cli/terminal".to_string(),
                },
                WorldImport {
                    name: "fs".to_string(),
                    interface: "wasi:filesystem/types".to_string(),
                },
            ],
            exports: vec![WorldExport {
                name: "handler".to_string(),
                interface: "ruchy:component/handler".to_string(),
            }],
            documentation: Some("My world documentation".to_string()),
        };
        let result = generator.format_world(&world);
        assert!(result.contains("/// My world documentation"));
        assert!(result.contains("world my-world"));
        assert!(result.contains("import console: wasi:cli/terminal"));
        assert!(result.contains("import fs: wasi:filesystem/types"));
        assert!(result.contains("export handler: ruchy:component/handler"));
    }

    #[test]
    fn test_format_world_empty() {
        let generator = WitGenerator::new();
        let world = WorldDefinition {
            name: "empty-world".to_string(),
            imports: vec![],
            exports: vec![],
            documentation: None,
        };
        let result = generator.format_world(&world);
        assert!(result.contains("world empty-world"));
        assert!(!result.contains("///"));
    }

    // WIT file generation tests
    #[test]
    fn test_generate_wit_file_full() {
        let generator = WitGenerator::new();
        let interface = WitInterface {
            name: "api".to_string(),
            version: "2.0.0".to_string(),
            package: PackageInfo {
                namespace: "myorg".to_string(),
                name: "mypackage".to_string(),
                version: "2.0.0".to_string(),
            },
            types: vec![TypeDefinition {
                name: "request".to_string(),
                kind: TypeKind::Record(vec![Field {
                    name: "path".to_string(),
                    field_type: WitType::String,
                    documentation: Some("Request path".to_string()),
                }]),
                documentation: Some("HTTP request".to_string()),
            }],
            functions: vec![FunctionDefinition {
                name: "handle".to_string(),
                params: vec![Parameter {
                    name: "req".to_string(),
                    param_type: WitType::Named("request".to_string()),
                }],
                return_type: Some(WitType::String),
                is_async: false,
                documentation: Some("Handle request".to_string()),
            }],
            resources: vec![],
            world: Some(WorldDefinition {
                name: "http".to_string(),
                imports: vec![],
                exports: vec![],
                documentation: None,
            }),
        };

        let wit_content = generator.generate_wit_file(&interface);
        assert!(wit_content.contains("package myorg:mypackage/api@2.0.0"));
        assert!(wit_content.contains("interface api"));
        assert!(wit_content.contains("/// HTTP request"));
        assert!(wit_content.contains("record request"));
        assert!(wit_content.contains("/// Request path"));
        assert!(wit_content.contains("path: string"));
        assert!(wit_content.contains("/// Handle request"));
        assert!(wit_content.contains("handle: func(req: request) -> string"));
        assert!(wit_content.contains("world http"));
    }

    #[test]
    fn test_generate_wit_file_no_world() {
        let generator = WitGenerator::new();
        let interface = WitInterface {
            name: "simple".to_string(),
            version: "1.0.0".to_string(),
            package: PackageInfo {
                namespace: "test".to_string(),
                name: "simple".to_string(),
                version: "1.0.0".to_string(),
            },
            types: vec![],
            functions: vec![],
            resources: vec![],
            world: None,
        };

        let wit_content = generator.generate_wit_file(&interface);
        assert!(wit_content.contains("package test:simple/simple@1.0.0"));
        assert!(wit_content.contains("interface simple"));
        assert!(!wit_content.contains("world"));
    }

    // WitInterface Display trait test
    #[test]
    fn test_wit_interface_display() {
        let interface = WitInterface {
            name: "display-test".to_string(),
            version: "1.0.0".to_string(),
            package: PackageInfo {
                namespace: "test".to_string(),
                name: "display".to_string(),
                version: "1.0.0".to_string(),
            },
            types: vec![],
            functions: vec![],
            resources: vec![],
            world: None,
        };

        let display = format!("{}", interface);
        assert!(display.contains("package test:display/display-test@1.0.0"));
        assert!(display.contains("interface display-test"));
    }

    // Generate methods tests
    #[test]
    fn test_generate_from_source() {
        let mut generator = WitGenerator::new();
        let result = generator.generate_from_source("fun main() {}");
        assert!(result.is_ok());
        let interface = result.unwrap();
        assert_eq!(interface.name, "ruchy-component");
        assert!(interface.types.len() > 0);
        assert!(interface.functions.len() > 0);
    }

    #[test]
    fn test_generate_from_component() {
        use crate::wasm::component::{ComponentMetadata, WasmComponent};

        let mut generator = WitGenerator::new();
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![],
            imports: vec![],
            exports: vec![],
            metadata: ComponentMetadata::default(),
            custom_sections: HashMap::new(),
        };

        let result = generator.generate_from_component(&component);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate() {
        use crate::wasm::component::{ComponentMetadata, WasmComponent};

        let mut generator = WitGenerator::new();
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![],
            imports: vec![],
            exports: vec![],
            metadata: ComponentMetadata::default(),
            custom_sections: HashMap::new(),
        };

        let result = generator.generate(&component);
        assert!(result.is_ok());
    }

    // MethodKind tests
    #[test]
    fn test_method_kind_constructor() {
        let kind = MethodKind::Constructor;
        assert_eq!(kind, MethodKind::Constructor);
    }

    #[test]
    fn test_method_kind_static() {
        let kind = MethodKind::Static;
        assert_eq!(kind, MethodKind::Static);
    }

    #[test]
    fn test_method_kind_instance() {
        let kind = MethodKind::Instance;
        assert_eq!(kind, MethodKind::Instance);
    }

    #[test]
    fn test_method_kind_clone() {
        let kind = MethodKind::Constructor;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_method_kind_debug() {
        let kind = MethodKind::Instance;
        let debug_str = format!("{:?}", kind);
        assert!(debug_str.contains("Instance"));
    }

    // ResourceMethod test
    #[test]
    fn test_resource_method_creation() {
        let method = ResourceMethod {
            name: "get-value".to_string(),
            kind: MethodKind::Instance,
            function: FunctionDefinition {
                name: "get-value".to_string(),
                params: vec![],
                return_type: Some(WitType::U32),
                is_async: false,
                documentation: None,
            },
        };

        assert_eq!(method.name, "get-value");
        assert_eq!(method.kind, MethodKind::Instance);
        assert_eq!(method.function.name, "get-value");
    }

    // Field tests
    #[test]
    fn test_field_creation() {
        let field = Field {
            name: "my-field".to_string(),
            field_type: WitType::U64,
            documentation: Some("A numeric field".to_string()),
        };

        assert_eq!(field.name, "my-field");
        assert!(matches!(field.field_type, WitType::U64));
        assert_eq!(field.documentation, Some("A numeric field".to_string()));
    }

    #[test]
    fn test_field_clone() {
        let field = Field {
            name: "test".to_string(),
            field_type: WitType::Bool,
            documentation: None,
        };
        let cloned = field.clone();
        assert_eq!(field.name, cloned.name);
    }

    // VariantCase tests
    #[test]
    fn test_variant_case_with_payload() {
        let case = VariantCase {
            name: "some-value".to_string(),
            payload: Some(WitType::String),
            documentation: Some("Has a value".to_string()),
        };

        assert_eq!(case.name, "some-value");
        assert!(case.payload.is_some());
        assert_eq!(case.documentation, Some("Has a value".to_string()));
    }

    #[test]
    fn test_variant_case_without_payload() {
        let case = VariantCase {
            name: "none".to_string(),
            payload: None,
            documentation: None,
        };

        assert_eq!(case.name, "none");
        assert!(case.payload.is_none());
    }

    // WorldImport and WorldExport tests
    #[test]
    fn test_world_import_creation() {
        let import = WorldImport {
            name: "logger".to_string(),
            interface: "wasi:logging/log".to_string(),
        };

        assert_eq!(import.name, "logger");
        assert_eq!(import.interface, "wasi:logging/log");
    }

    #[test]
    fn test_world_export_creation() {
        let export = WorldExport {
            name: "run".to_string(),
            interface: "wasi:cli/run".to_string(),
        };

        assert_eq!(export.name, "run");
        assert_eq!(export.interface, "wasi:cli/run");
    }

    // InterfaceDefinition test
    #[test]
    fn test_interface_definition_full() {
        let iface = InterfaceDefinition {
            name: "my-interface".to_string(),
            functions: vec![InterfaceFunction {
                name: "do-work".to_string(),
                params: vec![("input".to_string(), "string".to_string())],
                return_type: Some("u32".to_string()),
            }],
            types: vec![InterfaceType {
                name: "my-type".to_string(),
                definition: "record my-type {}".to_string(),
            }],
            documentation: Some("Interface documentation".to_string()),
        };

        assert_eq!(iface.name, "my-interface");
        assert_eq!(iface.functions.len(), 1);
        assert_eq!(iface.types.len(), 1);
        assert_eq!(
            iface.documentation,
            Some("Interface documentation".to_string())
        );
    }

    // InterfaceFunction with params test
    #[test]
    fn test_interface_function_with_params() {
        let func = InterfaceFunction {
            name: "transform".to_string(),
            params: vec![
                ("input".to_string(), "string".to_string()),
                ("options".to_string(), "my-options".to_string()),
            ],
            return_type: Some("result".to_string()),
        };

        assert_eq!(func.name, "transform");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].0, "input");
        assert_eq!(func.params[0].1, "string");
        assert_eq!(func.return_type, Some("result".to_string()));
    }

    // WitGenerator Default test
    #[test]
    fn test_wit_generator_default() {
        let generator = WitGenerator::default();
        assert!(generator.config.include_docs);
        assert!(generator.config.use_type_aliases);
    }

    // WitConfig serialize/deserialize tests
    #[test]
    fn test_wit_config_serialize() {
        let config = WitConfig::default();
        let json = serde_json::to_string(&config);
        assert!(json.is_ok());
    }

    #[test]
    fn test_wit_config_deserialize() {
        let json = r#"{"include_docs":true,"use_type_aliases":false,"generate_resources":true,"component_model_version":"0.2.0","type_mappings":{},"world_name":null}"#;
        let config: Result<WitConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.include_docs);
        assert!(!config.use_type_aliases);
    }

    // Clone/Debug derive tests
    #[test]
    fn test_wit_interface_clone() {
        let interface = WitInterface {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            package: PackageInfo {
                namespace: "t".to_string(),
                name: "t".to_string(),
                version: "1.0.0".to_string(),
            },
            types: vec![],
            functions: vec![],
            resources: vec![],
            world: None,
        };
        let cloned = interface.clone();
        assert_eq!(interface.name, cloned.name);
    }

    #[test]
    fn test_wit_interface_debug() {
        let interface = WitInterface {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            package: PackageInfo {
                namespace: "t".to_string(),
                name: "t".to_string(),
                version: "1.0.0".to_string(),
            },
            types: vec![],
            functions: vec![],
            resources: vec![],
            world: None,
        };
        let debug_str = format!("{:?}", interface);
        assert!(debug_str.contains("WitInterface"));
    }

    #[test]
    fn test_type_definition_clone() {
        let type_def = TypeDefinition {
            name: "test".to_string(),
            kind: TypeKind::Alias(Box::new(WitType::U32)),
            documentation: None,
        };
        let cloned = type_def.clone();
        assert_eq!(type_def.name, cloned.name);
    }

    #[test]
    fn test_function_definition_clone() {
        let func = FunctionDefinition {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            is_async: true,
            documentation: None,
        };
        let cloned = func.clone();
        assert_eq!(func.name, cloned.name);
        assert!(cloned.is_async);
    }

    #[test]
    fn test_resource_definition_clone() {
        let resource = ResourceDefinition {
            name: "res".to_string(),
            methods: vec![],
            constructor: None,
            documentation: None,
        };
        let cloned = resource.clone();
        assert_eq!(resource.name, cloned.name);
    }

    #[test]
    fn test_world_definition_clone() {
        let world = WorldDefinition {
            name: "w".to_string(),
            imports: vec![],
            exports: vec![],
            documentation: None,
        };
        let cloned = world.clone();
        assert_eq!(world.name, cloned.name);
    }

    #[test]
    fn test_type_kind_clone() {
        let kind = TypeKind::Flags(vec!["a".to_string(), "b".to_string()]);
        let cloned = kind.clone();
        if let TypeKind::Flags(flags) = cloned {
            assert_eq!(flags.len(), 2);
        } else {
            panic!("Expected Flags variant");
        }
    }

    #[test]
    fn test_wit_type_clone() {
        let wt = WitType::List(Box::new(WitType::U8));
        let cloned = wt.clone();
        assert!(matches!(cloned, WitType::List(_)));
    }

    // Resource with constructor test
    #[test]
    fn test_resource_definition_with_constructor() {
        let resource = ResourceDefinition {
            name: "connection".to_string(),
            methods: vec![ResourceMethod {
                name: "send".to_string(),
                kind: MethodKind::Instance,
                function: FunctionDefinition {
                    name: "send".to_string(),
                    params: vec![Parameter {
                        name: "data".to_string(),
                        param_type: WitType::List(Box::new(WitType::U8)),
                    }],
                    return_type: None,
                    is_async: false,
                    documentation: None,
                },
            }],
            constructor: Some(FunctionDefinition {
                name: "new".to_string(),
                params: vec![Parameter {
                    name: "host".to_string(),
                    param_type: WitType::String,
                }],
                return_type: None,
                is_async: false,
                documentation: Some("Create a new connection".to_string()),
            }),
            documentation: Some("Network connection resource".to_string()),
        };

        assert_eq!(resource.name, "connection");
        assert_eq!(resource.methods.len(), 1);
        assert!(resource.constructor.is_some());
        assert_eq!(
            resource.constructor.as_ref().unwrap().name,
            "new".to_string()
        );
    }

    // Async function test
    #[test]
    fn test_async_function_definition() {
        let func = FunctionDefinition {
            name: "fetch-data".to_string(),
            params: vec![Parameter {
                name: "url".to_string(),
                param_type: WitType::String,
            }],
            return_type: Some(WitType::Result {
                ok: Some(Box::new(WitType::List(Box::new(WitType::U8)))),
                err: Some(Box::new(WitType::String)),
            }),
            is_async: true,
            documentation: Some("Fetch data asynchronously".to_string()),
        };

        assert!(func.is_async);
        assert_eq!(func.name, "fetch-data");
    }

    // Empty WIT interface save error test
    #[test]
    fn test_wit_interface_save_error() {
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

        // Try to save to an invalid path (directory that doesn't exist)
        let result = interface.save("/nonexistent/path/to/file.wit");
        assert!(result.is_err());
    }

    // Package info serialization tests
    #[test]
    fn test_package_info_serialize() {
        let package = PackageInfo {
            namespace: "test".to_string(),
            name: "pkg".to_string(),
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&package);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"namespace\":\"test\""));
    }

    #[test]
    fn test_package_info_deserialize() {
        let json = r#"{"namespace":"myns","name":"mypkg","version":"2.0.0"}"#;
        let package: Result<PackageInfo, _> = serde_json::from_str(json);
        assert!(package.is_ok());
        let package = package.unwrap();
        assert_eq!(package.namespace, "myns");
        assert_eq!(package.name, "mypkg");
        assert_eq!(package.version, "2.0.0");
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
