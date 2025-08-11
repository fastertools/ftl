//! Spin manifest configuration file format (spin.toml)
//!
//! This module defines the complete Spin v3 manifest format with
//! type-safe validation using garde. This ensures robust transpilation
//! from ftl.toml to spin.toml.

use anyhow::{Context, Result};
use garde::Validate;
use serde::{Deserialize, Serialize, ser::SerializeMap};
use std::collections::HashMap;
use std::fmt::Write;
use toml_edit::{InlineTable, Item, Value};

/// Spin manifest version - must be 2 for Spin v2/v3
const SPIN_MANIFEST_VERSION: i64 = 2;

/// Root configuration structure for spin.toml
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SpinConfig {
    /// The version of the manifest format
    #[garde(range(min = 2, max = 2))]
    pub spin_manifest_version: i64,

    /// Application metadata
    #[garde(dive)]
    pub application: ApplicationConfig,

    /// Application-level variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(custom(validate_variables))]
    pub variables: HashMap<String, SpinVariable>,

    /// Components that make up the application
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(custom(validate_components))]
    pub component: HashMap<String, ComponentConfig>,
}

/// Application metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApplicationConfig {
    /// Name of the application
    #[garde(length(min = 1))]
    #[garde(pattern(r"^[a-zA-Z][a-zA-Z0-9_-]*$"))]
    pub name: String,

    /// Version of the application
    #[serde(default = "default_version")]
    #[garde(length(min = 1))]
    #[garde(pattern(r"^\d+\.\d+\.\d+$"))]
    pub version: String,

    /// Human-readable description
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[garde(skip)]
    pub description: String,

    /// List of authors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(skip)]
    pub authors: Vec<String>,

    /// Application-global trigger settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApplicationRedisTriggerConfig {
    /// Redis server address
    #[garde(length(min = 1))]
    #[garde(pattern(r"^redis://"))]
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ComponentConfig {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[garde(skip)]
    pub description: String,

    /// Source of the WebAssembly module
    #[garde(dive)]
    pub source: ComponentSource,

    /// Files to be made available to the component
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(skip)]
    pub files: Vec<FileMount>,

    /// Files to exclude
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(skip)]
    pub exclude_files: Vec<String>,

    /// Allowed outbound hosts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(custom(validate_outbound_hosts))]
    pub allowed_outbound_hosts: Vec<String>,

    /// Key-value stores the component can access
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(skip)]
    pub key_value_stores: Vec<String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
    pub environment: HashMap<String, String>,

    /// Build configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(dive)]
    pub build: Option<ComponentBuildConfig>,

    /// Component-specific variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
    pub variables: HashMap<String, String>,

    /// Whether dependencies inherit permissions
    #[serde(default)]
    #[garde(skip)]
    pub dependencies_inherit_configuration: bool,

    /// Component dependencies
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
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

impl Validate for ComponentSource {
    type Context = ();

    fn validate_into(
        &self,
        _ctx: &Self::Context,
        path: &mut dyn FnMut() -> garde::Path,
        report: &mut garde::Report,
    ) {
        match self {
            Self::Local(local_path) => {
                if local_path.is_empty() {
                    report.append(
                        path(),
                        garde::Error::new("Local source path cannot be empty"),
                    );
                }
            }
            Self::Remote { url, digest } => {
                if url.is_empty() {
                    report.append(
                        path(),
                        garde::Error::new("Remote source URL cannot be empty"),
                    );
                }
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    report.append(
                        path(),
                        garde::Error::new("Remote source URL must start with http:// or https://"),
                    );
                }
                // Check that URL has more than just the scheme
                if url == "http://" || url == "https://" {
                    report.append(
                        path(),
                        garde::Error::new("Remote source URL must include a host"),
                    );
                }
                if digest.is_empty() {
                    report.append(
                        path(),
                        garde::Error::new("Remote source digest cannot be empty"),
                    );
                }
                if !digest.starts_with("sha256:") {
                    report.append(
                        path(),
                        garde::Error::new("Remote source digest must start with sha256:"),
                    );
                }
            }
            Self::Registry {
                registry,
                package,
                version,
            } => {
                if registry.is_empty() {
                    report.append(path(), garde::Error::new("Registry domain cannot be empty"));
                }
                if package.is_empty() {
                    report.append(
                        path(),
                        garde::Error::new("Registry package cannot be empty"),
                    );
                }
                if !package.contains(':') {
                    report.append(
                        path(),
                        garde::Error::new("Registry package must be in format 'namespace:name'"),
                    );
                }
                if version.is_empty() {
                    report.append(
                        path(),
                        garde::Error::new("Registry version cannot be empty"),
                    );
                }
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ComponentBuildConfig {
    /// Build command to execute
    #[garde(length(min = 1))]
    pub command: String,

    /// Working directory
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[garde(skip)]
    pub workdir: String,

    /// Files to watch for changes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(skip)]
    pub watch: Vec<String>,

    /// Environment variables for build
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HttpTrigger {
    /// Route configuration
    #[garde(dive)]
    pub route: RouteConfig,

    /// Component to handle the trigger
    #[garde(length(min = 1))]
    pub component: String,

    /// Executor configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(dive)]
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

impl Validate for RouteConfig {
    type Context = ();

    fn validate_into(
        &self,
        _ctx: &Self::Context,
        path: &mut dyn FnMut() -> garde::Path,
        report: &mut garde::Report,
    ) {
        match self {
            Self::Path(route_path) => {
                if route_path.is_empty() && !route_path.is_empty() {
                    report.append(
                        path(),
                        garde::Error::new(
                            "Route path cannot be empty unless explicitly set to empty string",
                        ),
                    );
                }
            }
            Self::Private { private } => {
                if !private {
                    report.append(
                        path(),
                        garde::Error::new("Private route must have private: true"),
                    );
                }
            }
        }
    }
}

/// Executor configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
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
        #[garde(skip)]
        argv: Option<String>,
        /// Entry point function
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[garde(skip)]
        entrypoint: Option<String>,
    },
}

/// Redis trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RedisTrigger {
    /// Redis server address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub address: Option<String>,

    /// Channel to subscribe to
    #[garde(length(min = 1))]
    pub channel: String,

    /// Component to handle the trigger
    #[garde(length(min = 1))]
    pub component: String,
}

// Helper functions
fn default_version() -> String {
    "0.1.0".to_string()
}

// Custom validation functions
#[allow(clippy::trivially_copy_pass_by_ref)]
fn validate_variables(vars: &HashMap<String, SpinVariable>, _ctx: &()) -> garde::Result {
    for (name, var) in vars {
        // Validate variable names
        if name.is_empty() {
            return Err(garde::Error::new("Variable name cannot be empty"));
        }

        // Validate variable values
        match var {
            SpinVariable::Default { default }
            | SpinVariable::SecretDefault { default, secret: _ } => {
                if default.is_empty() {
                    // Empty default values are allowed
                }
            }
            SpinVariable::Required { required }
            | SpinVariable::SecretRequired {
                required,
                secret: _,
            } => {
                if !required {
                    return Err(garde::Error::new(format!(
                        "Variable '{name}' marked as required must have required: true"
                    )));
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn validate_components(components: &HashMap<String, ComponentConfig>, _ctx: &()) -> garde::Result {
    for (name, component) in components {
        // Validate component names (must be kebab-case and start with a letter)
        if name.is_empty() {
            return Err(garde::Error::new("Component name cannot be empty"));
        }

        // Must start with a letter (safe to check first char since we verified non-empty above)
        if let Some(first_char) = name.chars().next()
            && !first_char.is_alphabetic()
        {
            return Err(garde::Error::new(format!(
                "Component name '{name}' must start with a letter"
            )));
        }

        // Can only contain alphanumeric characters and hyphens
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(garde::Error::new(format!(
                "Component name '{name}' can only contain alphanumeric characters and hyphens"
            )));
        }

        // Validate each component
        component
            .validate()
            .map_err(|e| garde::Error::new(format!("Component '{name}': {e}")))?;
    }
    Ok(())
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn validate_outbound_hosts(hosts: &[String], _ctx: &()) -> garde::Result {
    for host in hosts {
        if host.is_empty() {
            return Err(garde::Error::new("Outbound host cannot be empty"));
        }

        // Allow template variables (e.g., {{ api_url }})
        if host.starts_with("{{") && host.ends_with("}}") {
            continue;
        }

        // Basic validation - must have scheme
        if !host.contains("://") {
            return Err(garde::Error::new(format!(
                "Outbound host '{host}' must include a scheme (e.g., http://, https://, redis://)"
            )));
        }
    }
    Ok(())
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

        // Validate using garde
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Validation error: {}", e))?;

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

        // Serialize components
        if !self.component.is_empty() {
            let mut component_table = toml_edit::Table::new();

            for (name, component) in &self.component {
                let comp_toml = toml::to_string(component)
                    .context(format!("Failed to serialize component {name}"))?;
                let comp_table: toml_edit::Table = comp_toml
                    .parse::<toml_edit::DocumentMut>()
                    .context(format!("Failed to parse component {name} table"))?
                    .as_table()
                    .clone();

                component_table[name] = Item::Table(comp_table);
            }

            doc["component"] = Item::Table(component_table);
        }

        // Convert to string
        let mut toml_string = doc.to_string();

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
    fn test_spin_config_validation() {
        // Valid minimal config
        let config = SpinConfig::new("test-app".to_string());
        assert!(config.validate().is_ok());

        // Test invalid manifest version
        let mut invalid_config = config.clone();
        invalid_config.spin_manifest_version = 1;
        assert!(invalid_config.validate().is_err());

        // Test invalid app name
        let invalid_config = SpinConfig::new("123-invalid".to_string());
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_component_source_validation() {
        // Valid local source
        let source = ComponentSource::Local("my-app.wasm".to_string());
        assert!(source.validate().is_ok());

        // Invalid empty local source
        let source = ComponentSource::Local(String::new());
        assert!(source.validate().is_err());

        // Valid remote source
        let source = ComponentSource::Remote {
            url: "https://example.com/app.wasm".to_string(),
            digest: "sha256:abcdef123456".to_string(),
        };
        assert!(source.validate().is_ok());

        // Invalid remote source (bad URL)
        let source = ComponentSource::Remote {
            url: "ftp://example.com/app.wasm".to_string(),
            digest: "sha256:abcdef123456".to_string(),
        };
        assert!(source.validate().is_err());

        // Valid registry source
        let source = ComponentSource::Registry {
            registry: "ghcr.io".to_string(),
            package: "myorg:myapp".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(source.validate().is_ok());

        // Invalid registry source (bad package format)
        let source = ComponentSource::Registry {
            registry: "ghcr.io".to_string(),
            package: "myorg-myapp".to_string(), // Missing colon
            version: "1.0.0".to_string(),
        };
        assert!(source.validate().is_err());
    }

    #[test]
    fn test_variable_validation() {
        let mut vars = HashMap::new();

        // Valid default variable
        vars.insert(
            "my_var".to_string(),
            SpinVariable::Default {
                default: "value".to_string(),
            },
        );
        assert!(validate_variables(&vars, &()).is_ok());

        // Valid required variable
        vars.insert(
            "required_var".to_string(),
            SpinVariable::Required { required: true },
        );
        assert!(validate_variables(&vars, &()).is_ok());

        // Invalid required variable (required: false)
        vars.insert(
            "bad_required".to_string(),
            SpinVariable::Required { required: false },
        );
        assert!(validate_variables(&vars, &()).is_err());
    }

    #[test]
    fn test_outbound_hosts_validation() {
        // Valid hosts
        let hosts = vec![
            "http://example.com".to_string(),
            "https://api.example.com:8080".to_string(),
            "redis://localhost:6379".to_string(),
            "*://example.com:*".to_string(),
        ];
        assert!(validate_outbound_hosts(&hosts, &()).is_ok());

        // Invalid host (no scheme)
        let hosts = vec!["example.com".to_string()];
        assert!(validate_outbound_hosts(&hosts, &()).is_err());

        // Empty host
        let hosts = vec![String::new()];
        assert!(validate_outbound_hosts(&hosts, &()).is_err());
    }
}

#[cfg(test)]
#[path = "spin_config_validation_tests.rs"]
mod validation_tests;
