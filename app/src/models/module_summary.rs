use serde::{Deserialize, Serialize};

/// Module component info for display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentInfo {
    pub component_type: String,
    pub name: Option<String>,
    pub number: String,
    pub rotation: String,
    pub sws: i32,
    pub language: String,
}

/// Simplified module data for list view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSummary {
    pub id: i32,
    pub version: i32,
    pub title: String,
    pub credits: i32,
    pub languages: Vec<String>,
    pub semester_rotations: Vec<String>,
    pub exam_categories: Vec<String>,
    pub study_programs: Vec<String>,
    pub study_program_ids: Vec<i32>,
    pub faculty_name: String,
    pub components: Vec<ComponentInfo>,
}

/// Search filters for module queries
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct SearchFilters {
    pub search_query: Option<String>,
    pub min_credits: Option<i32>,
    pub max_credits: Option<i32>,
    pub semester_rotations: Option<Vec<String>>,
    pub exam_categories: Option<Vec<String>>,
    pub study_program_ids: Option<Vec<i32>>,
    pub component_types: Option<Vec<String>>,
    pub component_languages: Option<Vec<String>>,
    pub starred_only: bool,
}

/// Options available for filters (populated from database)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterOptions {
    pub study_programs: Vec<StudyProgramOption>,
    pub exam_categories: Vec<String>,
    pub semester_rotations: Vec<String>,
    pub component_types: Vec<String>,
    pub component_languages: Vec<String>,
    pub credit_range: (i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudyProgramOption {
    pub id: i32,
    pub name: String,
}
