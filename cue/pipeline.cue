// SPDX-License-Identifier: PMPL-1.0-or-later
// pipeline.cue — CUE schema for .conflow.yaml pipeline definitions.
//
// This schema validates the structure of conflow pipeline files.
// It enforces required fields, allowed tool types, and correct stage wiring.

package conflow

// Pipeline is the top-level structure of a .conflow.yaml file.
#Pipeline: {
    version:     string | *"1"
    name:        string
    description?: string
    stages:      [...#Stage]
    env?: [string]: string
    cache?: #CacheConfig
}

// Stage represents a single execution step in the pipeline.
#Stage: {
    name:         string
    description?: string
    tool:         #Tool
    input:        #Input
    output?:      #Output
    depends_on?:  [...string]
    allow_failure?: bool | *false
    env?: [string]: string
    condition?: string
}

// Tool describes the config tool to invoke.
#Tool: #CueTool | #NickelTool | #ShellTool

#CueTool: {
    type:      "cue"
    command:   "vet" | "export" | "eval" | "def" | "trim"
    schemas?:  [...string]
    flags?:    [...string]
    out_format?: "json" | "yaml" | "text"
}

#NickelTool: {
    type:      "nickel"
    command:   "export" | "typecheck" | "query" | "pprint-ast"
    file?:     string
    flags?:    [...string]
    format?:   "json" | "yaml" | "toml" | "raw"
}

#ShellTool: {
    type:    "shell"
    command: string
    shell?:  "bash" | "sh" | "zsh" | "fish"
}

// Input describes what data flows into a stage.
#Input: string | {
    from_stage: string
} | [...string]

// Output describes where a stage writes its results.
#Output: string | {
    file: string
} | {
    stdout: bool | *true
}

// CacheConfig controls the pipeline-level content-addressed cache.
#CacheConfig: {
    enabled?:   bool | *true
    directory?: string | *".conflow-cache"
    max_size_mb?: number
}
