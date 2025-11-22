//! Type definitions for FAF format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete FAF file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FafData {
    pub faf_version: String,
    pub project: Project,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_score: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_confidence: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_tldr: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instant_context: Option<InstantContext>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_quality: Option<ContextQuality>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<Stack>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_context: Option<HumanContext>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Preferences>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub approach: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Instant context for AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_building: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_stack: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_files: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub commands: HashMap<String, String>,
}

/// Technical stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub infrastructure: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_tool: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cicd: Option<String>,
}

/// Context quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuality {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slots_filled: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,

    #[serde(default)]
    pub handoff_ready: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_context: Vec<String>,
}

/// Human context - the 6 W's
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,

    #[serde(rename = "why", skip_serializing_if = "Option::is_none")]
    pub why_field: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub how: Option<String>,

    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_field: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

/// Development preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_bar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_style: Option<String>,
}

/// Project state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub milestones: Vec<String>,
}
