// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! DAG (Directed Acyclic Graph) builder for pipeline dependencies
//!
//! Builds and validates dependency graphs for pipeline stages,
//! ensuring proper execution order and detecting cycles.

use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

use crate::errors::ConflowError;
use crate::pipeline::{Pipeline, Stage};

/// Builder for stage dependency DAGs
pub struct DagBuilder {
    graph: DiGraph<usize, ()>,
    name_to_index: HashMap<String, NodeIndex>,
    index_to_name: HashMap<NodeIndex, String>,
}

impl DagBuilder {
    /// Create a new DAG builder
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            name_to_index: HashMap::new(),
            index_to_name: HashMap::new(),
        }
    }

    /// Build a DAG from a pipeline
    pub fn build(pipeline: &Pipeline) -> Result<Self, ConflowError> {
        let mut builder = Self::new();

        // Add all stages as nodes
        for (idx, stage) in pipeline.stages.iter().enumerate() {
            let node = builder.graph.add_node(idx);
            builder.name_to_index.insert(stage.name.clone(), node);
            builder.index_to_name.insert(node, stage.name.clone());
        }

        // Add dependency edges
        for stage in &pipeline.stages {
            let stage_node = builder.name_to_index[&stage.name];

            // Explicit dependencies from depends_on
            for dep_name in &stage.depends_on {
                let dep_node = builder.name_to_index.get(dep_name).ok_or_else(|| {
                    ConflowError::UnknownDependency {
                        stage: stage.name.clone(),
                        dependency: dep_name.clone(),
                    }
                })?;

                builder.graph.add_edge(*dep_node, stage_node, ());
            }

            // Implicit dependencies from from_stage inputs
            if let Some(ref_stage) = stage.input.references_stage() {
                if let Some(dep_node) = builder.name_to_index.get(ref_stage) {
                    // Only add if not already present
                    if !builder.graph.contains_edge(*dep_node, stage_node) {
                        builder.graph.add_edge(*dep_node, stage_node, ());
                    }
                } else {
                    return Err(ConflowError::UnknownDependency {
                        stage: stage.name.clone(),
                        dependency: ref_stage.to_string(),
                    });
                }
            }
        }

        // Validate no cycles
        builder.validate_acyclic()?;

        Ok(builder)
    }

    /// Validate that the graph is acyclic
    fn validate_acyclic(&self) -> Result<(), ConflowError> {
        match toposort(&self.graph, None) {
            Ok(_) => Ok(()),
            Err(cycle) => {
                // Find cycle members
                let cycle_start = &self.index_to_name[&cycle.node_id()];
                let stages = self.find_cycle_members(cycle.node_id());
                Err(ConflowError::CircularDependency { stages })
            }
        }
    }

    /// Find all stages involved in a cycle
    fn find_cycle_members(&self, start: NodeIndex) -> Vec<String> {
        use petgraph::visit::{depth_first_search, DfsEvent};

        let mut in_cycle = vec![self.index_to_name[&start].clone()];
        let mut visited = std::collections::HashSet::new();

        // DFS to find cycle
        depth_first_search(&self.graph, Some(start), |event| {
            if let DfsEvent::Discover(node, _) = event {
                let name = &self.index_to_name[&node];
                if visited.contains(name) {
                    // Found cycle
                    in_cycle.push(name.clone());
                    return petgraph::visit::Control::Break(());
                }
                visited.insert(name.clone());
                in_cycle.push(name.clone());
            }
            petgraph::visit::Control::Continue
        });

        in_cycle
    }

    /// Get topologically sorted stage indices
    pub fn topological_order(&self) -> Result<Vec<usize>, ConflowError> {
        toposort(&self.graph, None)
            .map(|nodes| nodes.into_iter().map(|n| self.graph[n]).collect())
            .map_err(|cycle| {
                let stages = self.find_cycle_members(cycle.node_id());
                ConflowError::CircularDependency { stages }
            })
    }

    /// Get topologically sorted stage names
    pub fn topological_order_names(&self) -> Result<Vec<String>, ConflowError> {
        toposort(&self.graph, None)
            .map(|nodes| {
                nodes
                    .into_iter()
                    .map(|n| self.index_to_name[&n].clone())
                    .collect()
            })
            .map_err(|cycle| {
                let stages = self.find_cycle_members(cycle.node_id());
                ConflowError::CircularDependency { stages }
            })
    }

    /// Get dependencies for a stage (stages that must run before it)
    pub fn dependencies(&self, stage_name: &str) -> Option<Vec<String>> {
        let node = self.name_to_index.get(stage_name)?;
        let deps: Vec<String> = self
            .graph
            .neighbors_directed(*node, petgraph::Direction::Incoming)
            .map(|n| self.index_to_name[&n].clone())
            .collect();
        Some(deps)
    }

    /// Get dependents for a stage (stages that depend on it)
    pub fn dependents(&self, stage_name: &str) -> Option<Vec<String>> {
        let node = self.name_to_index.get(stage_name)?;
        let deps: Vec<String> = self
            .graph
            .neighbors_directed(*node, petgraph::Direction::Outgoing)
            .map(|n| self.index_to_name[&n].clone())
            .collect();
        Some(deps)
    }

    /// Check if stage A depends (directly or transitively) on stage B
    pub fn depends_on(&self, stage_a: &str, stage_b: &str) -> bool {
        let Some(node_a) = self.name_to_index.get(stage_a) else {
            return false;
        };
        let Some(node_b) = self.name_to_index.get(stage_b) else {
            return false;
        };

        // BFS from B to see if we can reach A
        petgraph::algo::has_path_connecting(&self.graph, *node_b, *node_a, None)
    }

    /// Generate Mermaid diagram of the DAG
    pub fn to_mermaid(&self) -> String {
        let mut out = String::from("graph TD\n");

        // Add nodes
        for (name, _) in &self.name_to_index {
            out.push_str(&format!("    {}[{}]\n", name, name));
        }

        // Add edges
        for edge in self.graph.edge_indices() {
            let (from, to) = self.graph.edge_endpoints(edge).unwrap();
            let from_name = &self.index_to_name[&from];
            let to_name = &self.index_to_name[&to];
            out.push_str(&format!("    {} --> {}\n", from_name, to_name));
        }

        out
    }

    /// Generate DOT diagram of the DAG
    pub fn to_dot(&self) -> String {
        let mut out = String::from("digraph pipeline {\n");
        out.push_str("    rankdir=TB;\n");
        out.push_str("    node [shape=box, style=rounded];\n\n");

        // Add edges (nodes are implicit)
        for edge in self.graph.edge_indices() {
            let (from, to) = self.graph.edge_endpoints(edge).unwrap();
            let from_name = &self.index_to_name[&from];
            let to_name = &self.index_to_name[&to];
            out.push_str(&format!("    \"{}\" -> \"{}\";\n", from_name, to_name));
        }

        // Add isolated nodes (no edges)
        for (name, node) in &self.name_to_index {
            if self.graph.neighbors_undirected(*node).count() == 0 {
                out.push_str(&format!("    \"{}\";\n", name));
            }
        }

        out.push_str("}\n");
        out
    }

    /// Generate text representation of execution order
    pub fn to_text(&self, pipeline: &Pipeline) -> Result<String, ConflowError> {
        let order = self.topological_order()?;
        let mut out = String::new();

        for (i, idx) in order.iter().enumerate() {
            let stage = &pipeline.stages[*idx];
            let deps = self.dependencies(&stage.name).unwrap_or_default();

            out.push_str(&format!("{}. {} ({})", i + 1, stage.name, stage.tool_name()));

            if !deps.is_empty() {
                out.push_str(&format!(" [depends: {}]", deps.join(", ")));
            }

            out.push('\n');
        }

        Ok(out)
    }
}

impl Default for DagBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{CueCommand, Input, Tool};

    fn make_test_pipeline(stages: Vec<(&str, Vec<&str>)>) -> Pipeline {
        Pipeline {
            version: "1".into(),
            name: "test".into(),
            description: None,
            stages: stages
                .into_iter()
                .map(|(name, deps)| Stage {
                    name: name.into(),
                    description: None,
                    tool: Tool::Cue {
                        command: CueCommand::Vet,
                        schemas: vec![],
                        flags: vec![],
                        out_format: None,
                    },
                    input: Input::Single("*.json".into()),
                    output: None,
                    depends_on: deps.into_iter().map(String::from).collect(),
                    allow_failure: false,
                    env: std::collections::HashMap::new(),
                    condition: None,
                })
                .collect(),
            env: std::collections::HashMap::new(),
            cache: crate::pipeline::CacheConfig::default(),
        }
    }

    #[test]
    fn test_linear_dag() {
        let pipeline = make_test_pipeline(vec![
            ("a", vec![]),
            ("b", vec!["a"]),
            ("c", vec!["b"]),
        ]);

        let dag = DagBuilder::build(&pipeline).unwrap();
        let order = dag.topological_order_names().unwrap();

        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_diamond_dag() {
        let pipeline = make_test_pipeline(vec![
            ("a", vec![]),
            ("b", vec!["a"]),
            ("c", vec!["a"]),
            ("d", vec!["b", "c"]),
        ]);

        let dag = DagBuilder::build(&pipeline).unwrap();
        let order = dag.topological_order_names().unwrap();

        // a must come first, d must come last
        assert_eq!(order[0], "a");
        assert_eq!(order[3], "d");
        // b and c can be in either order
        assert!(order[1] == "b" || order[1] == "c");
        assert!(order[2] == "b" || order[2] == "c");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let pipeline = make_test_pipeline(vec![("a", vec!["b"]), ("b", vec!["a"])]);

        let result = DagBuilder::build(&pipeline);
        assert!(matches!(result, Err(ConflowError::CircularDependency { .. })));
    }

    #[test]
    fn test_unknown_dependency() {
        let pipeline = make_test_pipeline(vec![("a", vec!["nonexistent"])]);

        let result = DagBuilder::build(&pipeline);
        assert!(matches!(result, Err(ConflowError::UnknownDependency { .. })));
    }

    #[test]
    fn test_depends_on_check() {
        let pipeline = make_test_pipeline(vec![
            ("a", vec![]),
            ("b", vec!["a"]),
            ("c", vec!["b"]),
        ]);

        let dag = DagBuilder::build(&pipeline).unwrap();

        assert!(dag.depends_on("c", "a")); // transitive
        assert!(dag.depends_on("c", "b")); // direct
        assert!(dag.depends_on("b", "a")); // direct
        assert!(!dag.depends_on("a", "c")); // reverse
        assert!(!dag.depends_on("a", "b")); // reverse
    }

    #[test]
    fn test_mermaid_output() {
        let pipeline = make_test_pipeline(vec![("a", vec![]), ("b", vec!["a"])]);

        let dag = DagBuilder::build(&pipeline).unwrap();
        let mermaid = dag.to_mermaid();

        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("a --> b"));
    }
}
