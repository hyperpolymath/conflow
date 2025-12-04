// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Init command - create a new conflow project

use colored::Colorize;
use miette::Result;
use std::path::Path;

/// Run the init command
pub async fn run(name: Option<String>, template: Option<String>, verbose: bool) -> Result<()> {
    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "my-project".to_string())
    });

    println!("{}", "Initializing conflow project...".bold());
    println!();

    // Check if .conflow.yaml already exists
    if Path::new(".conflow.yaml").exists() {
        return Err(miette::miette!(
            ".conflow.yaml already exists. Use --force to overwrite (not implemented yet)."
        ));
    }

    // Generate pipeline based on template
    let pipeline_content = match template.as_deref() {
        Some("cue-validation") => generate_cue_template(&project_name),
        Some("nickel-generation") => generate_nickel_template(&project_name),
        Some("full-pipeline") => generate_full_template(&project_name),
        Some("kubernetes") | Some("k8s") => generate_k8s_template(&project_name),
        Some(t) => {
            return Err(miette::miette!(
                "Unknown template: '{}'\n\nAvailable templates:\n\
                 • cue-validation     - Simple CUE schema validation\n\
                 • nickel-generation  - Nickel config generation\n\
                 • full-pipeline      - Generate → validate → export\n\
                 • kubernetes         - Kubernetes manifests pipeline",
                t
            ));
        }
        None => generate_default_template(&project_name),
    };

    // Write pipeline file
    std::fs::write(".conflow.yaml", &pipeline_content).map_err(|e| {
        miette::miette!("Failed to write .conflow.yaml: {}", e)
    })?;

    println!("  {} Created .conflow.yaml", "✓".green());

    // Create directories
    let dirs = [".conflow", "schemas", "configs"];
    for dir in dirs {
        if !Path::new(dir).exists() {
            std::fs::create_dir_all(dir).map_err(|e| {
                miette::miette!("Failed to create directory '{}': {}", dir, e)
            })?;
            println!("  {} Created {}/", "✓".green(), dir);
        }
    }

    // Create example files based on template
    if let Some(ref t) = template {
        create_example_files(t)?;
    }

    println!();
    println!("{}", "Project initialized!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to define your pipeline", ".conflow.yaml".cyan());
    println!("  2. Add your configuration files to {}", "configs/".cyan());
    println!("  3. Run {} to execute the pipeline", "conflow run".cyan());
    println!();

    if verbose {
        println!("{}", "Generated pipeline:".dimmed());
        println!("{}", "─".repeat(50).dimmed());
        println!("{}", pipeline_content.dimmed());
    }

    Ok(())
}

fn generate_default_template(name: &str) -> String {
    format!(
        r#"# conflow pipeline configuration
# Documentation: https://conflow.dev/docs/pipeline-reference

version: "1"
name: "{name}"

stages:
  - name: "validate"
    description: "Validate configuration files"
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/config.cue
    input: "configs/*.json"

# Add more stages as needed:
# - name: "export"
#   tool:
#     type: cue
#     command: export
#     out_format: yaml
#   input:
#     from_stage: validate
#   output: deploy/config.yaml
#   depends_on:
#     - validate
"#
    )
}

fn generate_cue_template(name: &str) -> String {
    format!(
        r#"# conflow pipeline - CUE validation
version: "1"
name: "{name}"

stages:
  - name: "validate"
    description: "Validate all configuration files against CUE schemas"
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/config.cue
    input: "configs/*.json"

  - name: "export-yaml"
    description: "Export validated config to YAML"
    tool:
      type: cue
      command: export
      out_format: yaml
    input: "configs/*.json"
    output: generated/config.yaml
    depends_on:
      - validate
"#
    )
}

fn generate_nickel_template(name: &str) -> String {
    format!(
        r#"# conflow pipeline - Nickel generation
version: "1"
name: "{name}"

stages:
  - name: "generate"
    description: "Generate configuration from Nickel"
    tool:
      type: nickel
      command: export
      file: configs/config.ncl
      format: json
    input: "configs/config.ncl"
    output: generated/config.json

  - name: "validate"
    description: "Validate generated configuration"
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/config.cue
    input:
      from_stage: generate
    depends_on:
      - generate
"#
    )
}

fn generate_full_template(name: &str) -> String {
    format!(
        r#"# conflow pipeline - Full generate → validate → export
version: "1"
name: "{name}"

stages:
  - name: "generate"
    description: "Generate configuration using Nickel"
    tool:
      type: nickel
      command: export
      file: configs/config.ncl
      format: json
    input: "configs/config.ncl"
    output: generated/config.json

  - name: "validate"
    description: "Validate against CUE schema"
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/config.cue
    input:
      from_stage: generate
    depends_on:
      - generate

  - name: "export"
    description: "Export final configuration as YAML"
    tool:
      type: cue
      command: export
      out_format: yaml
    input:
      from_stage: generate
    output: deploy/config.yaml
    depends_on:
      - validate
"#
    )
}

fn generate_k8s_template(name: &str) -> String {
    format!(
        r#"# conflow pipeline - Kubernetes manifests
version: "1"
name: "{name}"

stages:
  - name: "generate-base"
    description: "Generate base Kubernetes manifests"
    tool:
      type: nickel
      command: export
      file: k8s/base.ncl
      format: json
    input: "k8s/base.ncl"
    output: generated/base.json

  - name: "validate-k8s"
    description: "Validate Kubernetes resources"
    tool:
      type: cue
      command: vet
      schemas:
        - k8s/schemas/kubernetes.cue
    input:
      from_stage: generate-base
    depends_on:
      - generate-base

  - name: "export-manifests"
    description: "Export as YAML manifests"
    tool:
      type: cue
      command: export
      out_format: yaml
    input:
      from_stage: generate-base
    output: deploy/k8s.yaml
    depends_on:
      - validate-k8s
"#
    )
}

fn create_example_files(template: &str) -> Result<()> {
    match template {
        "cue-validation" => {
            // Create example CUE schema
            let schema = r#"// Configuration schema
package config

#Config: {
    name:     string & =~"^[a-z][a-z0-9-]*$"
    replicas: int & >=1 & <=10
    port:     int & >=1 & <=65535
    env:      "dev" | "staging" | "prod"

    resources?: {
        cpu?:    string
        memory?: string
    }
}
"#;
            std::fs::write("schemas/config.cue", schema)
                .map_err(|e| miette::miette!("Failed to write schema: {}", e))?;
            println!("  {} Created schemas/config.cue", "✓".green());

            // Create example config
            let config = r#"{
    "name": "my-app",
    "replicas": 3,
    "port": 8080,
    "env": "dev"
}
"#;
            std::fs::write("configs/example.json", config)
                .map_err(|e| miette::miette!("Failed to write example config: {}", e))?;
            println!("  {} Created configs/example.json", "✓".green());
        }
        "nickel-generation" | "full-pipeline" => {
            // Create example Nickel file
            let ncl = r#"# Configuration generation
{
  name = "my-app",
  replicas =
    let env = "dev" in
    if env == "prod" then 5
    else if env == "staging" then 3
    else 1,
  port = 8080,
  env = "dev",
}
"#;
            std::fs::write("configs/config.ncl", ncl)
                .map_err(|e| miette::miette!("Failed to write Nickel file: {}", e))?;
            println!("  {} Created configs/config.ncl", "✓".green());

            // Create CUE schema
            let schema = r#"// Configuration schema
package config

#Config: {
    name:     string
    replicas: int & >=1
    port:     int & >=1 & <=65535
    env:      string
}
"#;
            std::fs::write("schemas/config.cue", schema)
                .map_err(|e| miette::miette!("Failed to write schema: {}", e))?;
            println!("  {} Created schemas/config.cue", "✓".green());

            // Create generated directory
            std::fs::create_dir_all("generated")
                .map_err(|e| miette::miette!("Failed to create generated/: {}", e))?;
            std::fs::create_dir_all("deploy")
                .map_err(|e| miette::miette!("Failed to create deploy/: {}", e))?;
        }
        "kubernetes" | "k8s" => {
            // Create directories
            std::fs::create_dir_all("k8s/schemas")
                .map_err(|e| miette::miette!("Failed to create k8s/schemas/: {}", e))?;

            // Create base Nickel file
            let ncl = r#"# Kubernetes base configuration
{
  apiVersion = "apps/v1",
  kind = "Deployment",
  metadata = {
    name = "my-app",
    labels = {
      app = "my-app",
    },
  },
  spec = {
    replicas = 3,
    selector = {
      matchLabels = {
        app = "my-app",
      },
    },
    template = {
      metadata = {
        labels = {
          app = "my-app",
        },
      },
      spec = {
        containers = [
          {
            name = "my-app",
            image = "my-app:latest",
            ports = [
              { containerPort = 8080 },
            ],
          },
        ],
      },
    },
  },
}
"#;
            std::fs::write("k8s/base.ncl", ncl)
                .map_err(|e| miette::miette!("Failed to write k8s/base.ncl: {}", e))?;
            println!("  {} Created k8s/base.ncl", "✓".green());

            // Create K8s schema stub
            let schema = r#"// Kubernetes resource schema (simplified)
package kubernetes

#Deployment: {
    apiVersion: "apps/v1"
    kind:       "Deployment"
    metadata: {
        name:   string
        labels: [string]: string
    }
    spec: {
        replicas: int & >=0
        selector: matchLabels: [string]: string
        template: {
            metadata: labels: [string]: string
            spec: containers: [...#Container]
        }
    }
}

#Container: {
    name:  string
    image: string
    ports: [...{containerPort: int}]
}
"#;
            std::fs::write("k8s/schemas/kubernetes.cue", schema)
                .map_err(|e| miette::miette!("Failed to write k8s schema: {}", e))?;
            println!("  {} Created k8s/schemas/kubernetes.cue", "✓".green());

            std::fs::create_dir_all("generated")
                .map_err(|e| miette::miette!("Failed to create generated/: {}", e))?;
            std::fs::create_dir_all("deploy")
                .map_err(|e| miette::miette!("Failed to create deploy/: {}", e))?;
        }
        _ => {}
    }

    Ok(())
}
