use serde::{Deserialize, Serialize};

/// Intermediate structure for scraped module data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedModule {
    pub number: i32,
    pub version: i32,
    pub title: String,
    pub credits: i32,
    pub languages: Vec<String>,

    // Validity
    pub valid_since: Option<String>,
    pub valid_until: Option<String>,

    // Organizational
    pub faculty: Option<String>,
    pub institute: Option<String>,
    pub fachgebiet: Option<String>,
    pub responsible_person: Option<String>,
    pub examination_board: Option<String>,

    // Contact
    pub contact_email: Option<String>,
    pub contact_person: Option<String>,
    pub secretariat: Option<String>,
    pub website: Option<String>,

    // Content
    pub learning_result: Option<String>,
    pub content: Option<String>,
    pub teaching_information: Option<String>,
    pub requirements: Option<String>,
    pub additional_info: Option<String>,
    pub registration: Option<String>,
    pub max_attendees: Option<i32>,
    pub duration: Option<String>,

    // Components
    pub components: Vec<ScrapedComponent>,

    // Workload
    pub workload: Vec<ScrapedWorkload>,

    // Study programs
    pub study_programs: Vec<ScrapedStudyProgramUsage>,

    // IDs (might not always be present)
    pub m_pord_nr: Option<i32>,
    pub m_p_nr: Option<i32>,
    pub mp_pord_nr: Option<i32>,
    pub mp_p_nr: Option<i32>,

    pub moses_link: String,

    // Exam
    pub exam: Option<ScrapedExam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedExam {
    pub graded: bool,
    pub exam_type: String, // e.g., "Portfolioprüfung", "Mündliche Prüfung", "Schriftliche Prüfung"
    pub language: Option<String>,
    pub duration_scope: Option<String>,
    pub description: Option<String>,
    pub clef: Option<String>, // "Notenschlüssel"
    pub components: Vec<ScrapedExamComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedExamComponent {
    pub name: String,
    pub points: Option<i32>,
    pub category: Option<String>, // "oral", "written", or "Keine Angabe"
    pub scope: Option<String>, // "Dauer/Umfang"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedComponent {
    pub name: Option<String>,
    pub component_type: String, // VL, UE, PJ
    pub number: String,
    pub rotation: String, // WiSe, SoSe, WiSe/SoSe
    pub sws: i32,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedWorkload {
    pub description: String,
    pub factor: Option<f64>,
    pub hours: Option<f64>,
    pub total_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedStudyProgramUsage {
    pub study_program_name: String,
    pub study_program_link: Option<String>,
    pub stupo_name: String,
    pub stupo_link: Option<String>,
    pub first_usage: String,
    pub last_usage: String,
}
