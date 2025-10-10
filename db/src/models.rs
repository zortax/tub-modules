use serde::{Deserialize, Serialize};

// ============================================================================
// Enum Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::Type))]
#[cfg_attr(feature = "database", sqlx(type_name = "semester", rename_all = "PascalCase"))]
pub enum Semester {
    SoSe,
    WiSe,
}

// ExamType is now stored as TEXT in the database for flexibility
// Keeping the enum for backwards compatibility but it's no longer used in DB queries

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::Type))]
#[cfg_attr(feature = "database", sqlx(type_name = "exam_category", rename_all = "lowercase"))]
pub enum ExamCategory {
    Oral,
    Written,
    Praktisch,
}

// ComponentType is now stored as TEXT in the database for flexibility
// This allows for arbitrary component types like VL, UE, PJ, SEM, PR, etc.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::Type))]
#[cfg_attr(feature = "database", sqlx(type_name = "component_rotation"))]
pub enum ComponentRotation {
    WiSe,
    SoSe,
    #[serde(rename = "WiSe/SoSe")]
    #[cfg_attr(feature = "database", sqlx(rename = "WiSe/SoSe"))]
    WiSeSoSe,
}

// ============================================================================
// Table Structs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Module {
    pub id: i32,
    pub version: i32,
    pub valid_since_semester: Option<Semester>,
    pub valid_since_year: Option<i32>,
    pub valid_until_semester: Option<Semester>,
    pub valid_until_year: Option<i32>,
    pub languages: Vec<String>,
    pub title: String,
    pub credits: i32,
    pub m_pord_nr: Option<i32>,
    pub m_p_nr: Option<i32>,
    pub mp_pord_nr: Option<i32>,
    pub mp_p_nr: Option<i32>,
    pub faculty_id: i32,
    pub institute_id: i32,
    pub fg_id: i32,
    pub responsible_id: i32,
    pub examination_board_id: i32,
    pub learning_result: Option<String>,
    pub content: Option<String>,
    pub teaching_information: Option<String>,
    pub max_attendees: Option<i32>,
    pub registration: Option<String>,
    pub duration: Option<String>,
    pub requirements: Option<String>,
    pub additional_info: Option<String>,
    pub moses_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Faculty {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Institute {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Fachgebiet {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ResponsiblePerson {
    pub id: i32,
    pub name: String,
    pub fg_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Contact {
    pub id: i32,
    pub module_id: i32,
    pub module_version: i32,
    pub secretariat: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ExaminationBoard {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Exam {
    pub id: i32,
    pub module_id: i32,
    pub module_version: i32,
    pub graded: bool,
    pub exam_type: String,
    pub clef: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ExamComponent {
    pub id: i32,
    pub exam_id: i32,
    pub name: String,
    pub points: i32,
    pub category: ExamCategory,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ModuleComponent {
    pub id: i32,
    pub module_id: i32,
    pub module_version: i32,
    pub module_name: Option<String>,
    pub component_type: String,
    pub number: String,
    pub rotation: ComponentRotation,
    pub sws: i32,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ModuleWorkloadDistribution {
    pub id: i32,
    pub module_id: i32,
    pub module_version: i32,
    pub description: String,
    pub factor: f64,
    pub hours: f64,
    pub total_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct StudyProgram {
    pub id: i32,
    pub name: String,
    pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Stupo {
    pub id: i32,
    pub study_program_id: i32,
    pub name: String,
    pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ModuleCatalogUsage {
    pub id: i32,
    pub module_id: i32,
    pub module_version: i32,
    pub stupo_id: i32,
    pub first_usage: String,
    pub last_usage: String,
}
