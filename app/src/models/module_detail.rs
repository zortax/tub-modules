use super::module_summary::ComponentInfo;
use serde::{Deserialize, Serialize};

/// Complete module information for detail view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleDetail {
    // Basic info
    pub id: i32,
    pub version: i32,
    pub title: String,
    pub credits: i32,
    pub languages: Vec<String>,

    // Validity period
    pub valid_since: Option<String>,
    pub valid_until: Option<String>,

    // Organizations
    pub faculty: String,
    pub institute: String,
    pub fachgebiet: String,
    pub responsible_person: String,
    pub examination_board: String,

    // Contact
    pub contact: Option<ContactInfo>,

    // Content
    pub learning_result: Option<String>,
    pub content: Option<String>,
    pub teaching_information: Option<String>,
    pub requirements: Option<String>,
    pub additional_info: Option<String>,

    // Administrative
    pub registration: Option<String>,
    pub max_attendees: Option<i32>,
    pub duration: Option<String>,

    // Related data
    pub components: Vec<ComponentInfo>,
    pub exams: Vec<ExamInfo>,
    pub workload: Vec<WorkloadInfo>,
    pub study_programs: Vec<StudyProgramInfo>,

    // Moses link
    pub moses_link: String,
}

/// Contact information for a module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactInfo {
    pub secretariat: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

// ComponentInfo is defined in module_summary.rs and reused here

/// Exam information with components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExamInfo {
    pub id: i32,
    pub graded: bool,
    pub exam_type: String,
    pub description: Option<String>,
    pub components: Vec<ExamComponentInfo>,
}

/// Individual exam component
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExamComponentInfo {
    pub name: String,
    pub points: i32,
    pub category: String,
    pub scope: Option<String>,
}

/// Workload distribution entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkloadInfo {
    pub description: String,
    pub hours: f64,
}

/// Study program usage information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudyProgramInfo {
    pub program_name: String,
    pub stupo_name: String,
    pub first_usage: String,
    pub last_usage: String,
}
