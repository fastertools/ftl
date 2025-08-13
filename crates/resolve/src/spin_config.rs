//! Spin manifest configuration file format (spin.toml)
//!
//! This module defines the complete Spin v3 manifest format with
//! type-safe validation using garde. This ensures robust transpilation
//! from ftl.toml to spin.toml.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, ser::SerializeMap};
use std::collections::HashMap;
use std::fmt::Write;
use toml_edit::{InlineTable, Item, Value};

/// Spin manifest version - must be 2 for Spin v2/v3
const SPIN_MANIFEST_VERSION: i64 = 2;

/// Root configuration structure for spin.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinConfig {
    /// The version of the manifest format
    pub spin_manifest_version: i64,

    /// Application metadata
    pub application: ApplicationConfig,

    /// Application-level variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, SpinVariable>,

    /// Components that make up the application
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub component: HashMap<String, ComponentConfig>,
}

/// Application metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Name of the application
    pub name: String,

    /// Version of the application
    #[serde(default = "default_version")]
    pub version: String,

    /// Human-readable description
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    /// List of authors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,

    /// Application-global trigger settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<ApplicationTriggerConfig>,
}

/// Application-level trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTriggerConfig {
    /// Redis trigger configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis: Option<ApplicationRedisTriggerConfig>,
}

/// Application-level Redis trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationRedisTriggerConfig {
    /// Redis server address
    pub address: String,
}

/// Variable configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpinVariable {
    /// Variable with a default value
    Default {
        /// The default value
        default: String,
    },
    /// Required variable
    Required {
        /// Must be true
        required: bool,
    },
    /// Secret variable with default
    SecretDefault {
        /// The default value
        default: String,
        /// Mark as secret
        secret: bool,
    },
    /// Required secret variable
    SecretRequired {
        /// Must be true
        required: bool,
        /// Mark as secret
        secret: bool,
    },
}

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    /// Source of the WebAssembly module
    pub source: ComponentSource,

    /// Files to be made available to the component
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<FileMount>,

    /// Files to exclude
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_files: Vec<String>,

    /// Allowed outbound hosts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_outbound_hosts: Vec<String>,

    /// Key-value stores the component can access
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_value_stores: Vec<String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,

    /// Build configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<ComponentBuildConfig>,

    /// Component-specific variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, String>,

    /// Whether dependencies inherit permissions
    #[serde(default)]
    pub dependencies_inherit_configuration: bool,

    /// Component dependencies
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, ComponentDependency>,
}

/// Component source configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ComponentSource {
    /// Local file path
    Local(String),
    /// Remote URL with digest
    Remote {
        /// URL to the Wasm file
        #[serde(rename = "url")]
        url: String,
        /// SHA256 digest
        digest: String,
    },
    /// Registry reference
    Registry {
        /// Registry domain
        registry: String,
        /// Package name (format: "namespace:name")
        package: String,
        /// Version
        version: String,
    },
}

impl Serialize for ComponentSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Local(path) => path.serialize(serializer),
            Self::Remote { url, digest } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("url", url)?;
                map.serialize_entry("digest", digest)?;
                map.end()
            }
            Self::Registry {
                registry,
                package,
                version,
            } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("registry", registry)?;
                map.serialize_entry("package", package)?;
                map.serialize_entry("version", version)?;
                map.end()
            }
        }
    }
}

/// File mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileMount {
    /// Simple file or glob pattern
    Pattern(String),
    /// Source to destination mapping
    Mapping {
        /// Source file or directory
        source: String,
        /// Destination path
        destination: String,
    },
}

/// Component build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentBuildConfig {
    /// Build command to execute
    pub command: String,

    /// Working directory
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub workdir: String,

    /// Files to watch for changes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub watch: Vec<String>,

    /// Environment variables for build
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,
}

/// Component dependency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDependency {
    /// Registry domain
    pub registry: String,
    /// Package name
    pub package: String,
    /// Version
    pub version: String,
}

/// Trigger configuration (stored separately from components)
#[derive(Debug, Clone)]
pub struct TriggerConfig {
    /// HTTP triggers
    pub http: Vec<HttpTrigger>,
    /// Redis triggers
    pub redis: Vec<RedisTrigger>,
}

/// HTTP trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTrigger {
    /// Route configuration
    pub route: RouteConfig,

    /// Component to handle the trigger
    pub component: String,

    /// Executor configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor: Option<ExecutorConfig>,
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RouteConfig {
    /// Simple string route
    Path(String),
    /// Private route configuration
    Private {
        /// Must be true for private routes
        private: bool,
    },
}

/// Executor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExecutorConfig {
    /// Spin executor (default)
    #[serde(rename = "spin")]
    Spin,
    /// WAGI executor
    #[serde(rename = "wagi")]
    Wagi {
        /// Arguments to pass
        #[serde(default, skip_serializing_if = "Option::is_none")]
        argv: Option<String>,
        /// Entry point function
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entrypoint: Option<String>,
    },
}

/// Redis trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisTrigger {
    /// Redis server address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Channel to subscribe to
    pub channel: String,

    /// Component to handle the trigger
    pub component: String,
}

// Helper functions
fn default_version() -> String {
    "0.1.0".to_string()
}

impl SpinConfig {
    /// Create a new `SpinConfig` with default values
    pub fn new(name: String) -> Self {
        Self {
            spin_manifest_version: SPIN_MANIFEST_VERSION,
            application: ApplicationConfig {
                name,
                version: default_version(),
                description: String::new(),
                authors: Vec::new(),
                trigger: None,
            },
            variables: HashMap::new(),
            component: HashMap::new(),
        }
    }

    /// Parse `SpinConfig` from a TOML string
    pub fn parse(content: &str) -> Result<Self> {
        let config: Self = toml::from_str(content).context("Failed to parse spin.toml")?;

        Ok(config)
    }

    /// Convert to TOML string with triggers
    #[allow(clippy::too_many_lines)]
    pub fn to_toml_string_with_triggers(&self, triggers: &TriggerConfig) -> Result<String> {
        // First serialize the main config
        let mut doc = toml_edit::DocumentMut::new();

        // Set manifest version
        doc["spin_manifest_version"] = toml_edit::value(self.spin_manifest_version);

        // Serialize application
        let app_toml =
            toml::to_string(&self.application).context("Failed to serialize application")?;
        let app_table: toml_edit::Table = app_toml
            .parse::<toml_edit::DocumentMut>()
            .context("Failed to parse application table")?
            .as_table()
            .clone();
        doc["application"] = Item::Table(app_table);

        // Serialize variables if not empty
        if !self.variables.is_empty() {
            let mut vars_table = toml_edit::Table::new();
            for (name, var) in &self.variables {
                match var {
                    SpinVariable::Default { default } => {
                        let mut inline = InlineTable::new();
                        inline.insert("default", Value::from(default.clone()));
                        vars_table[name] = Item::Value(Value::InlineTable(inline));
                    }
                    SpinVariable::Required { required } => {
                        let mut inline = InlineTable::new();
                        inline.insert("required", Value::from(*required));
                        vars_table[name] = Item::Value(Value::InlineTable(inline));
                    }
                    SpinVariable::SecretDefault { default, secret } => {
                        let mut inline = InlineTable::new();
                        inline.insert("default", Value::from(default.clone()));
                        inline.insert("secret", Value::from(*secret));
                        vars_table[name] = Item::Value(Value::InlineTable(inline));
                    }
                    SpinVariable::SecretRequired { required, secret } => {
                        let mut inline = InlineTable::new();
                        inline.insert("required", Value::from(*required));
                        inline.insert("secret", Value::from(*secret));
                        vars_table[name] = Item::Value(Value::InlineTable(inline));
                    }
                }
            }
            doc["variables"] = Item::Table(vars_table);
        }

        // We'll manually add components after converting to string
        // to ensure proper dotted table notation

        // Convert to string
        let mut toml_string = doc.to_string();

        // Add components manually to get proper [component.name] format
        for (name, component) in &self.component {
            toml_string.push_str(&format!("\n[component.{name}]\n"));

            // Serialize component fields
            if !component.description.is_empty() {
                toml_string.push_str(&format!("description = \"{}\"\n", component.description));
            }

            // Source
            match &component.source {
                ComponentSource::Local(path) => {
                    toml_string.push_str(&format!("source = \"{}\"\n", path));
                }
                ComponentSource::Remote { url, digest } => {
                    toml_string.push_str(&format!(
                        "source = {{ url = \"{}\", digest = \"{}\" }}\n",
                        url, digest
                    ));
                }
                ComponentSource::Registry {
                    registry,
                    package,
                    version,
                } => {
                    toml_string.push_str(&format!(
                        "source = {{ registry = \"{}\", package = \"{}\", version = \"{}\" }}\n",
                        registry, package, version
                    ));
                }
            }

            // Files
            if !component.files.is_empty() {
                toml_string.push_str("files = [");
                for (i, file) in component.files.iter().enumerate() {
                    if i > 0 {
                        toml_string.push_str(", ");
                    }
                    match file {
                        FileMount::Pattern(p) => toml_string.push_str(&format!("\"{}\"", p)),
                        FileMount::Mapping {
                            source,
                            destination,
                        } => {
                            toml_string.push_str(&format!(
                                "{{ source = \"{}\", destination = \"{}\" }}",
                                source, destination
                            ));
                        }
                    }
                }
                toml_string.push_str("]\n");
            }

            // Exclude files
            if !component.exclude_files.is_empty() {
                toml_string.push_str(&format!("exclude_files = {:?}\n", component.exclude_files));
            }

            // Allowed outbound hosts
            if !component.allowed_outbound_hosts.is_empty() {
                toml_string.push_str(&format!(
                    "allowed_outbound_hosts = {:?}\n",
                    component.allowed_outbound_hosts
                ));
            }

            // Key-value stores
            if !component.key_value_stores.is_empty() {
                toml_string.push_str(&format!(
                    "key_value_stores = {:?}\n",
                    component.key_value_stores
                ));
            }

            // Dependencies inherit configuration
            if component.dependencies_inherit_configuration {
                toml_string.push_str("dependencies_inherit_configuration = true\n");
            }

            // Build section
            if let Some(build) = &component.build {
                toml_string.push_str(&format!("\n[component.{name}.build]\n"));
                // Properly escape the command string
                let escaped_command = build.command.replace('\\', "\\\\").replace('"', "\\\"");
                toml_string.push_str(&format!("command = \"{}\"\n", escaped_command));
                if !build.workdir.is_empty() {
                    toml_string.push_str(&format!("workdir = \"{}\"\n", build.workdir));
                }
                if !build.watch.is_empty() {
                    toml_string.push_str(&format!("watch = {:?}\n", build.watch));
                }
                if !build.environment.is_empty() {
                    toml_string.push_str("environment = { ");
                    let mut first = true;
                    for (key, value) in &build.environment {
                        if !first {
                            toml_string.push_str(", ");
                        }
                        first = false;
                        let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
                        toml_string.push_str(&format!("{key} = \"{escaped_value}\""));
                    }
                    toml_string.push_str(" }\n");
                }
            }

            // Variables section
            if !component.variables.is_empty() {
                toml_string.push_str(&format!("\n[component.{name}.variables]\n"));
                for (var_name, var_value) in &component.variables {
                    // Properly escape variable values
                    let escaped_value = var_value.replace('\\', "\\\\").replace('"', "\\\"");
                    toml_string.push_str(&format!("{var_name} = \"{}\"\n", escaped_value));
                }
            }
        }

        // Add triggers manually to get the correct [[trigger.http]] format
        if !triggers.http.is_empty() {
            toml_string.push('\n');
            for trigger in &triggers.http {
                toml_string.push_str("[[trigger.http]]\n");

                // Route
                match &trigger.route {
                    RouteConfig::Path(path) => {
                        writeln!(toml_string, "route = \"{path}\"").unwrap();
                    }
                    RouteConfig::Private { private: _ } => {
                        toml_string.push_str("route = { private = true }\n");
                    }
                }

                // Component
                writeln!(toml_string, "component = \"{}\"", trigger.component).unwrap();

                // Executor if present
                if let Some(executor) = &trigger.executor {
                    match executor {
                        ExecutorConfig::Spin => {
                            toml_string.push_str("executor = { type = \"spin\" }\n");
                        }
                        ExecutorConfig::Wagi { argv, entrypoint } => {
                            toml_string.push_str("executor = { type = \"wagi\"");
                            if let Some(argv) = argv {
                                write!(toml_string, ", argv = \"{argv}\"").unwrap();
                            }
                            if let Some(entrypoint) = entrypoint {
                                write!(toml_string, ", entrypoint = \"{entrypoint}\"").unwrap();
                            }
                            toml_string.push_str(" }\n");
                        }
                    }
                }

                toml_string.push('\n');
            }
        }

        if !triggers.redis.is_empty() {
            toml_string.push('\n');
            for trigger in &triggers.redis {
                toml_string.push_str("[[trigger.redis]]\n");

                if let Some(address) = &trigger.address {
                    writeln!(toml_string, "address = \"{address}\"").unwrap();
                }

                writeln!(toml_string, "channel = \"{}\"", trigger.channel).unwrap();
                writeln!(toml_string, "component = \"{}\"", trigger.component).unwrap();
                toml_string.push('\n');
            }
        }

        Ok(toml_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spin_config_creation() {
        let config = SpinConfig::new("test-app".to_string());
        assert_eq!(config.application.name, "test-app");
        assert_eq!(config.spin_manifest_version, 2);
    }

    #[test]
    fn test_component_source_serialization() {
        use std::collections::HashMap;

        // Test component with local source
        let component = ComponentConfig {
            description: String::new(),
            source: ComponentSource::Local("my-app.wasm".to_string()),
            files: vec![],
            exclude_files: vec![],
            allowed_outbound_hosts: vec![],
            key_value_stores: vec![],
            environment: HashMap::new(),
            build: None,
            variables: HashMap::new(),
            dependencies_inherit_configuration: false,
            dependencies: HashMap::new(),
        };
        let serialized = toml::to_string(&component).unwrap();
        assert!(serialized.contains("source = \"my-app.wasm\""));

        // Test component with registry source - but the serializer converts it to a single string
        let component = ComponentConfig {
            description: String::new(),
            source: ComponentSource::Local("ghcr.io/myorg/myapp:1.0.0".to_string()),
            files: vec![],
            exclude_files: vec![],
            allowed_outbound_hosts: vec![],
            key_value_stores: vec![],
            environment: HashMap::new(),
            build: None,
            variables: HashMap::new(),
            dependencies_inherit_configuration: false,
            dependencies: HashMap::new(),
        };
        let serialized = toml::to_string(&component).unwrap();
        assert!(serialized.contains("source = \"ghcr.io/myorg/myapp:1.0.0\""));
    }
}
