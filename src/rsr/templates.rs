// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Template generation for compliant configurations
//!
//! Generate RSR-compliant configuration structures from templates.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ConflowError;

/// Template type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateType {
    /// Simple CUE validation pipeline
    CueValidation,
    /// Nickel generation pipeline
    NickelGeneration,
    /// Full pipeline (generate, validate, export)
    FullPipeline,
    /// Multi-environment configuration
    MultiEnv,
    /// Kubernetes configuration
    Kubernetes,
    /// Terraform configuration
    Terraform,
    /// Helm chart
    Helm,
    /// Docker Compose
    DockerCompose,
    /// Custom template
    Custom,
}

impl TemplateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CueValidation => "cue-validation",
            Self::NickelGeneration => "nickel-generation",
            Self::FullPipeline => "full-pipeline",
            Self::MultiEnv => "multi-env",
            Self::Kubernetes => "kubernetes",
            Self::Terraform => "terraform",
            Self::Helm => "helm",
            Self::DockerCompose => "docker-compose",
            Self::Custom => "custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::CueValidation => "Simple CUE schema validation",
            Self::NickelGeneration => "Programmatic config generation with Nickel",
            Self::FullPipeline => "Generate, validate, and export pipeline",
            Self::MultiEnv => "Multi-environment configuration management",
            Self::Kubernetes => "Kubernetes manifest validation",
            Self::Terraform => "Terraform configuration validation",
            Self::Helm => "Helm chart configuration",
            Self::DockerCompose => "Docker Compose configuration",
            Self::Custom => "Custom template",
        }
    }
}

/// Template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template name
    pub name: String,

    /// Template type
    pub template_type: TemplateType,

    /// Description
    pub description: String,

    /// Files to generate
    pub files: Vec<TemplateFile>,

    /// Directories to create
    pub directories: Vec<String>,

    /// Variables that can be customized
    pub variables: HashMap<String, TemplateVariable>,
}

/// A file in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    /// Target path (relative to project root)
    pub path: String,

    /// File content
    pub content: String,

    /// Whether this file should be overwritten if it exists
    #[serde(default)]
    pub overwrite: bool,
}

/// A variable in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable description
    pub description: String,

    /// Default value
    pub default: String,

    /// Whether this variable is required
    #[serde(default)]
    pub required: bool,
}

/// Template generator
pub struct TemplateGenerator {
    templates: HashMap<String, Template>,
}

impl TemplateGenerator {
    /// Create a new template generator
    pub fn new() -> Self {
        let mut generator = Self {
            templates: HashMap::new(),
        };

        generator.register_builtin_templates();
        generator
    }

    /// Register built-in templates
    fn register_builtin_templates(&mut self) {
        // CUE Validation template
        self.templates.insert(
            "cue-validation".into(),
            Template {
                name: "cue-validation".into(),
                template_type: TemplateType::CueValidation,
                description: "Simple CUE schema validation".into(),
                directories: vec!["schemas".into(), "config".into()],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_CUE_VALIDATION_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "schemas/config.cue".into(),
                        content: TEMPLATE_CUE_SCHEMA.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "config/example.yaml".into(),
                        content: TEMPLATE_EXAMPLE_CONFIG.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::from([
                    (
                        "project_name".into(),
                        TemplateVariable {
                            description: "Project name".into(),
                            default: "my-project".into(),
                            required: true,
                        },
                    ),
                ]),
            },
        );

        // Nickel Generation template
        self.templates.insert(
            "nickel-generation".into(),
            Template {
                name: "nickel-generation".into(),
                template_type: TemplateType::NickelGeneration,
                description: "Programmatic config generation with Nickel".into(),
                directories: vec!["nickel".into(), "dist".into()],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_NICKEL_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "nickel/config.ncl".into(),
                        content: TEMPLATE_NICKEL_CONFIG.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::from([
                    (
                        "project_name".into(),
                        TemplateVariable {
                            description: "Project name".into(),
                            default: "my-project".into(),
                            required: true,
                        },
                    ),
                ]),
            },
        );

        // Full Pipeline template
        self.templates.insert(
            "full-pipeline".into(),
            Template {
                name: "full-pipeline".into(),
                template_type: TemplateType::FullPipeline,
                description: "Generate, validate, and export pipeline".into(),
                directories: vec!["schemas".into(), "nickel".into(), "dist".into()],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_FULL_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "schemas/config.cue".into(),
                        content: TEMPLATE_CUE_SCHEMA.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "nickel/config.ncl".into(),
                        content: TEMPLATE_NICKEL_CONFIG.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::new(),
            },
        );

        // Multi-environment template
        self.templates.insert(
            "multi-env".into(),
            Template {
                name: "multi-env".into(),
                template_type: TemplateType::MultiEnv,
                description: "Multi-environment configuration management".into(),
                directories: vec![
                    "environments".into(),
                    "schemas".into(),
                    "dist".into(),
                ],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_MULTI_ENV_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "environments/base.ncl".into(),
                        content: TEMPLATE_MULTI_ENV_BASE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "environments/development.ncl".into(),
                        content: TEMPLATE_MULTI_ENV_DEV.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "environments/staging.ncl".into(),
                        content: TEMPLATE_MULTI_ENV_STAGING.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "environments/production.ncl".into(),
                        content: TEMPLATE_MULTI_ENV_PROD.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::new(),
            },
        );

        // Kubernetes template
        self.templates.insert(
            "kubernetes".into(),
            Template {
                name: "kubernetes".into(),
                template_type: TemplateType::Kubernetes,
                description: "Kubernetes manifest validation".into(),
                directories: vec!["k8s".into(), "schemas".into()],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_K8S_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "schemas/k8s.cue".into(),
                        content: TEMPLATE_K8S_SCHEMA.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "k8s/deployment.yaml".into(),
                        content: TEMPLATE_K8S_DEPLOYMENT.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::from([
                    (
                        "app_name".into(),
                        TemplateVariable {
                            description: "Application name".into(),
                            default: "my-app".into(),
                            required: true,
                        },
                    ),
                ]),
            },
        );

        // Terraform template
        self.templates.insert(
            "terraform".into(),
            Template {
                name: "terraform".into(),
                template_type: TemplateType::Terraform,
                description: "Terraform configuration validation".into(),
                directories: vec!["terraform".into(), "schemas".into()],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_TERRAFORM_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "schemas/tfvars.cue".into(),
                        content: TEMPLATE_TERRAFORM_SCHEMA.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::new(),
            },
        );

        // Helm template
        self.templates.insert(
            "helm".into(),
            Template {
                name: "helm".into(),
                template_type: TemplateType::Helm,
                description: "Helm chart configuration".into(),
                directories: vec!["chart".into(), "schemas".into()],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_HELM_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "schemas/values.cue".into(),
                        content: TEMPLATE_HELM_VALUES_SCHEMA.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "chart/values.yaml".into(),
                        content: TEMPLATE_HELM_VALUES.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::new(),
            },
        );

        // Docker Compose template
        self.templates.insert(
            "docker-compose".into(),
            Template {
                name: "docker-compose".into(),
                template_type: TemplateType::DockerCompose,
                description: "Docker Compose configuration".into(),
                directories: vec!["schemas".into()],
                files: vec![
                    TemplateFile {
                        path: ".conflow.yaml".into(),
                        content: TEMPLATE_COMPOSE_PIPELINE.into(),
                        overwrite: false,
                    },
                    TemplateFile {
                        path: "schemas/compose.cue".into(),
                        content: TEMPLATE_COMPOSE_SCHEMA.into(),
                        overwrite: false,
                    },
                ],
                variables: HashMap::new(),
            },
        );
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// List all templates
    pub fn list(&self) -> impl Iterator<Item = &Template> {
        self.templates.values()
    }

    /// Generate template files in target directory
    pub fn generate(
        &self,
        template_name: &str,
        target_dir: &Path,
        variables: &HashMap<String, String>,
    ) -> Result<GenerationResult, ConflowError> {
        let template = self.get(template_name).ok_or_else(|| ConflowError::ExecutionFailed {
            message: format!("Template not found: {}", template_name),
            help: Some(format!(
                "Available templates: {}",
                self.templates.keys().cloned().collect::<Vec<_>>().join(", ")
            )),
        })?;

        let mut result = GenerationResult {
            template_name: template_name.to_string(),
            files_created: vec![],
            files_skipped: vec![],
            directories_created: vec![],
        };

        // Create directories
        for dir in &template.directories {
            let path = target_dir.join(dir);
            if !path.exists() {
                std::fs::create_dir_all(&path)?;
                result.directories_created.push(dir.clone());
            }
        }

        // Generate files
        for file in &template.files {
            let path = target_dir.join(&file.path);

            if path.exists() && !file.overwrite {
                result.files_skipped.push(file.path.clone());
                continue;
            }

            // Apply variable substitution
            let content = self.substitute_variables(&file.content, variables);

            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&path, content)?;
            result.files_created.push(file.path.clone());
        }

        Ok(result)
    }

    /// Substitute variables in content
    fn substitute_variables(&self, content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = content.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{ {} }}}}", key);
            result = result.replace(&placeholder, value);

            // Also support ${var} syntax
            let alt_placeholder = format!("${{{}}}", key);
            result = result.replace(&alt_placeholder, value);
        }

        result
    }

    /// Register a custom template
    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Load templates from a directory
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize, ConflowError> {
        if !dir.exists() {
            return Ok(0);
        }

        let mut count = 0;

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = std::fs::read_to_string(&path)?;
                let template: Template = serde_yaml::from_str(&content).map_err(|e| {
                    ConflowError::Yaml {
                        message: e.to_string(),
                    }
                })?;

                self.templates.insert(template.name.clone(), template);
                count += 1;
            }
        }

        Ok(count)
    }
}

impl Default for TemplateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of template generation
#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub template_name: String,
    pub files_created: Vec<String>,
    pub files_skipped: Vec<String>,
    pub directories_created: Vec<String>,
}

// Template content strings

const TEMPLATE_CUE_VALIDATION_PIPELINE: &str = r#"# CUE Validation Pipeline
# Generated by conflow

version: "1"
name: {{ project_name }}

stages:
  - name: validate
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/config.cue
    input:
      - "config/*.yaml"
      - "config/*.json"
    description: Validate configuration against CUE schema

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_CUE_SCHEMA: &str = r#"// Configuration Schema
package config

#Config: {
    // Application version
    version: string & =~"^[0-9]+\\.[0-9]+\\.[0-9]+$"

    // Application name
    name: string & !=""

    // Environment
    environment?: "development" | "staging" | "production"

    // Logging configuration
    logging?: {
        level: "debug" | "info" | "warn" | "error"
        format?: "json" | "text"
    }

    // Feature flags
    features?: [string]: bool
}
"#;

const TEMPLATE_EXAMPLE_CONFIG: &str = r#"# Example configuration
version: "1.0.0"
name: my-application
environment: development
logging:
  level: info
  format: json
features:
  new_ui: false
  analytics: true
"#;

const TEMPLATE_NICKEL_PIPELINE: &str = r#"# Nickel Generation Pipeline
# Generated by conflow

version: "1"
name: {{ project_name }}

stages:
  - name: generate
    tool:
      type: nickel
      command: export
      format: yaml
    input: nickel/config.ncl
    output: dist/config.yaml
    description: Generate configuration from Nickel

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_NICKEL_CONFIG: &str = r#"# Configuration in Nickel
{
  version = "1.0.0",
  name = "{{ project_name }}",

  # Default settings
  environment = "development",
  debug = false,
  log_level = "info",

  # Database configuration
  database = {
    host = "localhost",
    port = 5432,
    name = "app_db",
    pool_size = 10,
  },

  # Feature flags
  features = {
    new_ui = false,
    analytics = true,
  },
}
"#;

const TEMPLATE_FULL_PIPELINE: &str = r#"# Full Pipeline: Generate, Validate, Export
# Generated by conflow

version: "1"
name: config-pipeline

stages:
  # Generate config from Nickel
  - name: generate
    tool:
      type: nickel
      command: export
      format: yaml
    input: nickel/config.ncl
    output: dist/config.yaml
    description: Generate configuration from Nickel

  # Validate generated config
  - name: validate
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/config.cue
    input: dist/config.yaml
    depends_on:
      - generate
    description: Validate configuration against schema

  # Export as JSON
  - name: export-json
    tool:
      type: shell
      command: "yq -o json dist/config.yaml > dist/config.json"
    depends_on:
      - validate
    description: Export as JSON

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_MULTI_ENV_PIPELINE: &str = r#"# Multi-Environment Pipeline
# Generated by conflow

version: "1"
name: multi-env-config

stages:
  - name: generate-dev
    tool:
      type: nickel
      command: export
      format: yaml
    input: environments/development.ncl
    output: dist/config.development.yaml
    description: Generate development config

  - name: generate-staging
    tool:
      type: nickel
      command: export
      format: yaml
    input: environments/staging.ncl
    output: dist/config.staging.yaml
    description: Generate staging config

  - name: generate-production
    tool:
      type: nickel
      command: export
      format: yaml
    input: environments/production.ncl
    output: dist/config.production.yaml
    description: Generate production config

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_MULTI_ENV_BASE: &str = r#"# Base configuration
# Override these values per environment
{
  app_name = "my-application",
  version = "1.0.0",

  log_level = "info",
  debug = false,

  features = {
    new_ui = false,
    analytics = true,
  },

  database = {
    host = "localhost",
    port = 5432,
    pool_size = 10,
  },
}
"#;

const TEMPLATE_MULTI_ENV_DEV: &str = r#"# Development environment
let base = import "base.ncl" in

base & {
  environment = "development",
  debug = true,
  log_level = "debug",
}
"#;

const TEMPLATE_MULTI_ENV_STAGING: &str = r#"# Staging environment
let base = import "base.ncl" in

base & {
  environment = "staging",
  log_level = "info",
  features.new_ui = true,
  database.host = "staging-db.internal",
}
"#;

const TEMPLATE_MULTI_ENV_PROD: &str = r#"# Production environment
let base = import "base.ncl" in

base & {
  environment = "production",
  log_level = "warn",
  database = base.database & {
    host = "prod-db.internal",
    pool_size = 50,
  },
}
"#;

const TEMPLATE_K8S_PIPELINE: &str = r#"# Kubernetes Validation Pipeline
# Generated by conflow

version: "1"
name: k8s-validation

stages:
  - name: validate
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/k8s.cue
    input:
      - "k8s/*.yaml"
    description: Validate Kubernetes manifests

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_K8S_SCHEMA: &str = r#"// Kubernetes Schema
package k8s

#Deployment: {
    apiVersion: "apps/v1"
    kind:       "Deployment"
    metadata:   #Metadata
    spec:       #DeploymentSpec
}

#Metadata: {
    name:        string & !=""
    namespace?:  string
    labels?:     [string]: string
    annotations?: [string]: string
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
    name:  string & !=""
    image: string & !=""
    ports?: [...#ContainerPort]
    resources?: #Resources
}

#ContainerPort: {
    containerPort: int & >=1 & <=65535
    protocol?:     "TCP" | "UDP"
}

#Resources: {
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

const TEMPLATE_K8S_DEPLOYMENT: &str = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ app_name }}
  labels:
    app: {{ app_name }}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {{ app_name }}
  template:
    metadata:
      name: {{ app_name }}
      labels:
        app: {{ app_name }}
    spec:
      containers:
        - name: {{ app_name }}
          image: {{ app_name }}:latest
          ports:
            - containerPort: 8080
          resources:
            limits:
              cpu: "100m"
              memory: "128Mi"
            requests:
              cpu: "50m"
              memory: "64Mi"
"#;

const TEMPLATE_TERRAFORM_PIPELINE: &str = r#"# Terraform Validation Pipeline
# Generated by conflow

version: "1"
name: terraform-validation

stages:
  - name: validate-vars
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/tfvars.cue
    input:
      - "terraform/*.tfvars.json"
    description: Validate Terraform variables

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_TERRAFORM_SCHEMA: &str = r#"// Terraform Variables Schema
package terraform

#Variables: {
    // AWS region
    region: string

    // Environment name
    environment: "dev" | "staging" | "prod"

    // Instance type
    instance_type?: string | *"t3.micro"

    // Enable monitoring
    monitoring?: bool | *true

    // Tags
    tags?: [string]: string
}
"#;

const TEMPLATE_HELM_PIPELINE: &str = r#"# Helm Values Validation Pipeline
# Generated by conflow

version: "1"
name: helm-validation

stages:
  - name: validate-values
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/values.cue
    input:
      - "chart/values.yaml"
    description: Validate Helm values

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_HELM_VALUES_SCHEMA: &str = r#"// Helm Values Schema
package helm

#Values: {
    // Replica count
    replicaCount: int & >=1

    // Image configuration
    image: {
        repository: string
        tag:        string | *"latest"
        pullPolicy: "Always" | "IfNotPresent" | "Never"
    }

    // Service configuration
    service: {
        type: "ClusterIP" | "NodePort" | "LoadBalancer"
        port: int & >=1 & <=65535
    }

    // Resource limits
    resources?: {
        limits?: {
            cpu?:    string
            memory?: string
        }
        requests?: {
            cpu?:    string
            memory?: string
        }
    }
}
"#;

const TEMPLATE_HELM_VALUES: &str = r#"# Default values
replicaCount: 1

image:
  repository: nginx
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 80

resources:
  limits:
    cpu: 100m
    memory: 128Mi
  requests:
    cpu: 50m
    memory: 64Mi
"#;

const TEMPLATE_COMPOSE_PIPELINE: &str = r#"# Docker Compose Validation Pipeline
# Generated by conflow

version: "1"
name: compose-validation

stages:
  - name: validate
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/compose.cue
    input:
      - "docker-compose*.yaml"
    description: Validate Docker Compose configuration

cache:
  enabled: true
  directory: .conflow-cache
"#;

const TEMPLATE_COMPOSE_SCHEMA: &str = r#"// Docker Compose Schema
package compose

#Compose: {
    version?: string
    services: [string]: #Service
    volumes?: [string]: #Volume
    networks?: [string]: #Network
}

#Service: {
    image?:       string
    build?:       string | #Build
    ports?:       [...string]
    environment?: [...string] | [string]: string
    volumes?:     [...string]
    depends_on?:  [...string]
    networks?:    [...string]
    restart?:     "no" | "always" | "on-failure" | "unless-stopped"
}

#Build: {
    context:    string
    dockerfile?: string
    args?:       [string]: string
}

#Volume: {
    driver?: string
    external?: bool
}

#Network: {
    driver?: string
    external?: bool
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_templates() {
        let generator = TemplateGenerator::new();
        let templates: Vec<_> = generator.list().collect();

        assert!(templates.len() >= 6);
        assert!(templates.iter().any(|t| t.name == "cue-validation"));
        assert!(templates.iter().any(|t| t.name == "kubernetes"));
    }

    #[test]
    fn test_generate_template() {
        let temp = TempDir::new().unwrap();
        let generator = TemplateGenerator::new();

        let mut variables = HashMap::new();
        variables.insert("project_name".to_string(), "test-project".to_string());

        let result = generator
            .generate("cue-validation", temp.path(), &variables)
            .unwrap();

        assert!(!result.files_created.is_empty());
        assert!(temp.path().join(".conflow.yaml").exists());
        assert!(temp.path().join("schemas/config.cue").exists());

        // Check variable substitution
        let content = std::fs::read_to_string(temp.path().join(".conflow.yaml")).unwrap();
        assert!(content.contains("test-project"));
    }

    #[test]
    fn test_generate_kubernetes_template() {
        let temp = TempDir::new().unwrap();
        let generator = TemplateGenerator::new();

        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "my-app".to_string());

        let result = generator
            .generate("kubernetes", temp.path(), &variables)
            .unwrap();

        assert!(!result.files_created.is_empty());
        assert!(temp.path().join("k8s/deployment.yaml").exists());

        let content = std::fs::read_to_string(temp.path().join("k8s/deployment.yaml")).unwrap();
        assert!(content.contains("my-app"));
    }
}
