//! RSR Schema Registry
//!
//! Provides access to RSR schemas for validation and generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ConflowError;

/// Schema type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    /// CUE schema
    Cue,
    /// JSON Schema
    JsonSchema,
    /// Nickel contract
    Nickel,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    /// Schema ID
    pub id: String,

    /// Schema type
    pub schema_type: SchemaType,

    /// Schema name
    pub name: String,

    /// Description
    pub description: String,

    /// Schema content (inline) or path
    pub source: SchemaSource,

    /// Version
    pub version: String,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Schema source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaSource {
    /// Inline schema content
    Inline { content: String },

    /// Path to schema file
    Path { path: PathBuf },

    /// URL to fetch schema
    Url { url: String },
}

/// RSR Schema Registry
pub struct RsrSchemaRegistry {
    schemas: HashMap<String, SchemaDefinition>,
    cache_dir: Option<PathBuf>,
}

impl RsrSchemaRegistry {
    /// Create a new schema registry
    pub fn new() -> Self {
        let mut registry = Self {
            schemas: HashMap::new(),
            cache_dir: None,
        };

        // Register built-in schemas
        registry.register_builtins();

        registry
    }

    /// Create with cache directory
    pub fn with_cache(cache_dir: PathBuf) -> Self {
        let mut registry = Self::new();
        registry.cache_dir = Some(cache_dir);
        registry
    }

    /// Register built-in RSR schemas
    fn register_builtins(&mut self) {
        // RSR Pipeline Schema
        self.schemas.insert(
            "rsr:pipeline".into(),
            SchemaDefinition {
                id: "rsr:pipeline".into(),
                schema_type: SchemaType::Cue,
                name: "RSR Pipeline Schema".into(),
                description: "Schema for .conflow.yaml pipeline definitions".into(),
                source: SchemaSource::Inline {
                    content: include_str!("../../cue/pipeline.cue").into(),
                },
                version: "1.0.0".into(),
                tags: vec!["conflow".into(), "pipeline".into()],
            },
        );

        // RSR Requirement Schema
        self.schemas.insert(
            "rsr:requirement".into(),
            SchemaDefinition {
                id: "rsr:requirement".into(),
                schema_type: SchemaType::Cue,
                name: "RSR Requirement Schema".into(),
                description: "Schema for RSR requirement definitions".into(),
                source: SchemaSource::Inline {
                    content: RSR_REQUIREMENT_SCHEMA.into(),
                },
                version: "1.0.0".into(),
                tags: vec!["rsr".into(), "requirement".into()],
            },
        );

        // RSR Config Schema
        self.schemas.insert(
            "rsr:config".into(),
            SchemaDefinition {
                id: "rsr:config".into(),
                schema_type: SchemaType::Cue,
                name: "RSR Configuration Schema".into(),
                description: "Schema for .rsr.yaml configuration files".into(),
                source: SchemaSource::Inline {
                    content: RSR_CONFIG_SCHEMA.into(),
                },
                version: "1.0.0".into(),
                tags: vec!["rsr".into(), "config".into()],
            },
        );

        // Kubernetes base schema
        self.schemas.insert(
            "k8s:base".into(),
            SchemaDefinition {
                id: "k8s:base".into(),
                schema_type: SchemaType::Cue,
                name: "Kubernetes Base Schema".into(),
                description: "Base schema for Kubernetes resources".into(),
                source: SchemaSource::Inline {
                    content: K8S_BASE_SCHEMA.into(),
                },
                version: "1.0.0".into(),
                tags: vec!["kubernetes".into(), "k8s".into()],
            },
        );
    }

    /// Get a schema by ID
    pub fn get(&self, id: &str) -> Option<&SchemaDefinition> {
        self.schemas.get(id)
    }

    /// Get schema content
    pub fn get_content(&self, id: &str) -> Result<String, ConflowError> {
        let schema = self.schemas.get(id).ok_or_else(|| ConflowError::FileNotFound {
            path: PathBuf::from(id),
            help: Some("Schema not found in registry".into()),
        })?;

        match &schema.source {
            SchemaSource::Inline { content } => Ok(content.clone()),
            SchemaSource::Path { path } => {
                std::fs::read_to_string(path).map_err(|e| ConflowError::Io {
                    message: e.to_string(),
                })
            }
            SchemaSource::Url { url } => {
                // Would fetch from URL
                Err(ConflowError::ExecutionFailed {
                    message: format!("URL schemas not yet implemented: {}", url),
                    help: None,
                })
            }
        }
    }

    /// List all schemas
    pub fn list(&self) -> impl Iterator<Item = &SchemaDefinition> {
        self.schemas.values()
    }

    /// List schemas by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&SchemaDefinition> {
        self.schemas
            .values()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Register a custom schema
    pub fn register(&mut self, schema: SchemaDefinition) {
        self.schemas.insert(schema.id.clone(), schema);
    }

    /// Load schemas from a directory
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize, ConflowError> {
        let mut count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(dir).map_err(|e| ConflowError::Io {
            message: e.to_string(),
        })? {
            let entry = entry.map_err(|e| ConflowError::Io {
                message: e.to_string(),
            })?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = std::fs::read_to_string(&path).map_err(|e| ConflowError::Io {
                    message: e.to_string(),
                })?;

                let schema: SchemaDefinition =
                    serde_yaml::from_str(&content).map_err(|e| ConflowError::Yaml {
                        message: e.to_string(),
                    })?;

                self.schemas.insert(schema.id.clone(), schema);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Write schema to file
    pub fn write_to_file(&self, id: &str, path: &Path) -> Result<(), ConflowError> {
        let content = self.get_content(id)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConflowError::Io {
                message: e.to_string(),
            })?;
        }

        std::fs::write(path, content).map_err(|e| ConflowError::Io {
            message: e.to_string(),
        })?;

        Ok(())
    }
}

impl Default for RsrSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in schema definitions

const RSR_REQUIREMENT_SCHEMA: &str = r#"
// RSR Requirement Schema
package rsr

#Requirement: {
    id:          string & =~"^RSR-[A-Z]+-[0-9]+$"
    name:        string
    class:       "mandatory" | "preferential" | "advisory"
    description: string

    validation: {
        file_exists?:   [...string]
        file_absent?:   [...string]
        patterns?:      [...#PatternCheck]
        cue_validate?:  [...#CueValidation]
        conflow_valid?: bool
        shell_check?:   string
    }

    remediation: {
        auto_fix?:      bool
        templates?:     [...#Template]
        manual_steps?:  [...string]
        docs_url?:      string
    }

    related?: [...string]
    tags?:    [...string]
}

#PatternCheck: {
    file:         string
    pattern:      string
    should_match: bool | *true
}

#CueValidation: {
    files:  [...string]
    schema: string
}

#Template: {
    name:             string
    description:      string
    conflow_template?: string
    generates?:       [...string]
}
"#;

const RSR_CONFIG_SCHEMA: &str = r#"
// RSR Configuration Schema
// For .rsr.yaml files
package rsr

#Config: {
    // RSR version
    version: "1" | *"1"

    // Project metadata
    project: {
        name:        string
        description?: string
        tier?:       1 | 2 | 3 | 4
    }

    // Requirements configuration
    requirements?: {
        // Skip specific requirements
        skip?: [...string]

        // Custom requirement definitions
        custom?: [...#Requirement]
    }

    // Integration settings
    integrations?: {
        conflow?: {
            enabled: bool | *true
            pipeline?: string
        }

        ci?: {
            provider?: "github" | "gitlab" | "jenkins"
            config?:   string
        }
    }

    // Compliance targets
    compliance?: {
        target_level?: "basic" | "good" | "excellent"
        exceptions?:   [...{
            requirement: string
            reason:      string
            expires?:    string
        }]
    }
}
"#;

const K8S_BASE_SCHEMA: &str = r#"
// Kubernetes Base Schema
package k8s

#Resource: {
    apiVersion: string
    kind:       string
    metadata:   #Metadata
}

#Metadata: {
    name:        string
    namespace?:  string
    labels?:     [string]: string
    annotations?: [string]: string
}

#Deployment: #Resource & {
    apiVersion: "apps/v1"
    kind:       "Deployment"
    spec:       #DeploymentSpec
}

#DeploymentSpec: {
    replicas?: int & >=0
    selector:  #Selector
    template:  #PodTemplateSpec
}

#Selector: {
    matchLabels: [string]: string
}

#PodTemplateSpec: {
    metadata: #Metadata
    spec:     #PodSpec
}

#PodSpec: {
    containers: [...#Container]
}

#Container: {
    name:  string
    image: string
    ports?: [...#ContainerPort]
    env?:   [...#EnvVar]
    resources?: #ResourceRequirements
}

#ContainerPort: {
    containerPort: int & >=1 & <=65535
    protocol?:     "TCP" | "UDP" | *"TCP"
}

#EnvVar: {
    name:  string
    value?: string
    valueFrom?: {
        secretKeyRef?: {
            name: string
            key:  string
        }
        configMapKeyRef?: {
            name: string
            key:  string
        }
    }
}

#ResourceRequirements: {
    limits?: {
        cpu?:    string
        memory?: string
    }
    requests?: {
        cpu?:    string
        memory?: string
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_builtins() {
        let registry = RsrSchemaRegistry::new();

        assert!(registry.get("rsr:pipeline").is_some());
        assert!(registry.get("rsr:requirement").is_some());
        assert!(registry.get("rsr:config").is_some());
        assert!(registry.get("k8s:base").is_some());
    }

    #[test]
    fn test_get_content() {
        let registry = RsrSchemaRegistry::new();

        let content = registry.get_content("rsr:pipeline").unwrap();
        assert!(content.contains("#Pipeline"));
    }

    #[test]
    fn test_by_tag() {
        let registry = RsrSchemaRegistry::new();

        let rsr_schemas = registry.by_tag("rsr");
        assert!(rsr_schemas.len() >= 2);
    }
}
