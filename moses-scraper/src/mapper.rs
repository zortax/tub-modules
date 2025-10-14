use anyhow::{Context, Result};
use sqlx::PgPool;

use crate::models::{ScrapedModule, ScrapedComponent, ScrapedStudyProgramUsage, ScrapedExam};

pub struct MappedModuleData {
    pub module: ModuleData,
    pub contact: Option<ContactData>,
    pub components: Vec<ComponentData>,
    pub workload: Vec<WorkloadData>,
    pub study_program_usages: Vec<StudyProgramUsageData>,
    pub exam: Option<ExamData>,
    pub exam_components: Vec<ExamComponentData>,
}

pub struct ModuleData {
    pub id: i32,
    pub version: i32,
    pub scraping_run_id: i32,
    pub valid_since_semester: Option<db::Semester>,
    pub valid_since_year: Option<i32>,
    pub valid_until_semester: Option<db::Semester>,
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

pub struct ContactData {
    pub module_id: i32,
    pub module_version: i32,
    pub module_scraping_run_id: i32,
    pub secretariat: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

pub struct ComponentData {
    pub module_id: i32,
    pub module_version: i32,
    pub module_scraping_run_id: i32,
    pub module_name: Option<String>,
    pub component_type: String,
    pub number: String,
    pub rotation: db::ComponentRotation,
    pub sws: i32,
    pub language: String,
}

pub struct WorkloadData {
    pub module_id: i32,
    pub module_version: i32,
    pub module_scraping_run_id: i32,
    pub description: String,
    pub factor: f64,
    pub hours: f64,
    pub total_hours: f64,
}

pub struct StudyProgramUsageData {
    pub module_id: i32,
    pub module_version: i32,
    pub module_scraping_run_id: i32,
    pub stupo_id: i32,
    pub first_usage: String,
    pub last_usage: String,
}

pub struct ExamData {
    pub module_id: i32,
    pub module_version: i32,
    pub module_scraping_run_id: i32,
    pub graded: bool,
    pub exam_type: String,
    pub clef: Option<String>,
    pub description: Option<String>,
}

pub struct ExamComponentData {
    pub name: String,
    pub points: i32,
    pub category: db::ExamCategory,
    pub scope: Option<String>,
}

pub async fn map_module_data(pool: &PgPool, scraped: ScrapedModule, scraping_run_id: i32) -> Result<MappedModuleData> {
    // Get or create reference data
    let faculty_id = get_or_create_faculty(pool, &scraped.faculty.unwrap_or_else(|| "Unknown".to_string())).await?;
    let institute_id = get_or_create_institute(pool, &scraped.institute.unwrap_or_else(|| "Unknown".to_string())).await?;
    let fg_id = get_or_create_fachgebiet(pool, &scraped.fachgebiet.unwrap_or_else(|| "Unknown".to_string())).await?;
    let examination_board_id = get_or_create_examination_board(pool, &scraped.examination_board.unwrap_or_else(|| "Unknown".to_string())).await?;
    let responsible_id = get_or_create_responsible(pool, &scraped.responsible_person.unwrap_or_else(|| "Unknown".to_string()), Some(fg_id)).await?;

    // Parse validity period (optional)
    let (valid_since_semester, valid_since_year) = if let Some(ref since) = scraped.valid_since {
        match parse_validity_period(&Some(since.clone())) {
            Ok((sem, year)) => (Some(sem), Some(year)),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    let (valid_until_semester, valid_until_year) = if let Some(ref until) = scraped.valid_until {
        match parse_validity_period(&Some(until.clone())) {
            Ok((sem, year)) => (Some(sem), Some(year)),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    // Extract languages
    let languages = if scraped.languages.is_empty() {
        vec!["de".to_string()]
    } else {
        scraped.languages
    };

    // Build module data
    let module = ModuleData {
        id: scraped.number,
        version: scraped.version,
        scraping_run_id,
        valid_since_semester,
        valid_since_year,
        valid_until_semester,
        valid_until_year,
        languages,
        title: scraped.title,
        credits: scraped.credits,
        m_pord_nr: scraped.m_pord_nr,
        m_p_nr: scraped.m_p_nr,
        mp_pord_nr: scraped.mp_pord_nr,
        mp_p_nr: scraped.mp_p_nr,
        faculty_id,
        institute_id,
        fg_id,
        responsible_id,
        examination_board_id,
        learning_result: scraped.learning_result,
        content: scraped.content,
        teaching_information: scraped.teaching_information,
        max_attendees: scraped.max_attendees,
        registration: scraped.registration,
        duration: scraped.duration,
        requirements: scraped.requirements,
        additional_info: scraped.additional_info,
        moses_link: scraped.moses_link,
    };

    // Build contact data
    let contact = if scraped.contact_email.is_some() || scraped.contact_person.is_some() || scraped.secretariat.is_some() || scraped.website.is_some() {
        Some(ContactData {
            module_id: scraped.number,
            module_version: scraped.version,
            module_scraping_run_id: scraping_run_id,
            secretariat: scraped.secretariat,
            contact_person: scraped.contact_person,
            email: scraped.contact_email,
            website: scraped.website,
        })
    } else {
        None
    };

    // Map components
    let components = scraped.components.into_iter()
        .filter_map(|c| map_component(scraped.number, scraped.version, scraping_run_id, c).ok())
        .collect();

    // Map workload
    let workload = scraped.workload.into_iter()
        .map(|w| WorkloadData {
            module_id: scraped.number,
            module_version: scraped.version,
            module_scraping_run_id: scraping_run_id,
            description: w.description,
            factor: w.factor.unwrap_or(1.0),
            hours: w.hours.unwrap_or(0.0),
            total_hours: w.total_hours,
        })
        .collect();

    // Map study program usages
    let mut study_program_usages = Vec::new();
    for usage in scraped.study_programs {
        match map_study_program_usage(pool, scraped.number, scraped.version, scraping_run_id, usage).await {
            Ok(mapped) => study_program_usages.push(mapped),
            Err(_e) => {
                // Skip failed study program mappings
            },
        }
    }

    // Map exam
    let (exam, exam_components) = if let Some(scraped_exam) = scraped.exam {
        match map_exam(scraped.number, scraped.version, scraping_run_id, scraped_exam) {
            Ok((exam_data, components_data)) => (Some(exam_data), components_data),
            Err(_e) => (None, Vec::new()),
        }
    } else {
        (None, Vec::new())
    };

    Ok(MappedModuleData {
        module,
        contact,
        components,
        workload,
        study_program_usages,
        exam,
        exam_components,
    })
}

fn parse_validity_period(period_str: &Option<String>) -> Result<(db::Semester, i32)> {
    let period = period_str.as_ref().context("No validity period provided")?;

    // Try to extract semester and year from formats like:
    // "WiSe 2018" or "Wintersemester 2018/2019" or "SoSe 2018" or "Sommersemester 2018"

    let semester = if period.contains("SoSe") || period.contains("Sommer") || period.contains("Summer") {
        db::Semester::SoSe
    } else {
        db::Semester::WiSe
    };

    // Extract year
    let year = period
        .split_whitespace()
        .find_map(|word| {
            // Handle formats like "2018/2019" by taking the first year
            let year_str = word.split('/').next().unwrap_or(word);
            year_str.parse::<i32>().ok()
        })
        .context("Failed to parse year from validity period")?;

    Ok((semester, year))
}

fn map_component(module_id: i32, module_version: i32, scraping_run_id: i32, component: ScrapedComponent) -> Result<ComponentData> {
    // Normalize component type to uppercase, handle special characters
    let component_type = match component.component_type.to_uppercase().as_str() {
        "ÜE" => "UE".to_string(),
        other => other.to_string()
    };

    let rotation = match component.rotation.as_str() {
        "WiSe" | "Wintersemester" => db::ComponentRotation::WiSe,
        "SoSe" | "Sommersemester" => db::ComponentRotation::SoSe,
        "WiSe/SoSe" | "jedes Semester" => db::ComponentRotation::WiSeSoSe,
        _ => db::ComponentRotation::SoSe
    };

    Ok(ComponentData {
        module_id,
        module_version,
        module_scraping_run_id: scraping_run_id,
        module_name: component.name,
        component_type,
        number: component.number,
        rotation,
        sws: component.sws,
        language: component.language,
    })
}

async fn map_study_program_usage(pool: &PgPool, module_id: i32, module_version: i32, scraping_run_id: i32, usage: ScrapedStudyProgramUsage) -> Result<StudyProgramUsageData> {
    // Get or create study program
    let study_program_id = get_or_create_study_program(
        pool,
        &usage.study_program_name,
        &usage.study_program_link.unwrap_or_else(|| "https://www.tu-berlin.de".to_string())
    ).await?;

    // Get or create stupo
    let stupo_id = get_or_create_stupo(
        pool,
        study_program_id,
        &usage.stupo_name,
        &usage.stupo_link.unwrap_or_else(|| "https://www.tu-berlin.de".to_string())
    ).await?;

    Ok(StudyProgramUsageData {
        module_id,
        module_version,
        module_scraping_run_id: scraping_run_id,
        stupo_id,
        first_usage: usage.first_usage,
        last_usage: usage.last_usage,
    })
}

// Database helper functions

async fn get_or_create_faculty(pool: &PgPool, name: &str) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO faculty (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id",
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

async fn get_or_create_institute(pool: &PgPool, name: &str) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO institute (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id",
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

async fn get_or_create_fachgebiet(pool: &PgPool, name: &str) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO fachgebiet (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id",
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

async fn get_or_create_examination_board(pool: &PgPool, name: &str) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO examination_board (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id",
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

async fn get_or_create_responsible(pool: &PgPool, name: &str, fg_id: Option<i32>) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO responsible_person (name, fg_id) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET fg_id = $2 RETURNING id",
        name,
        fg_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

async fn get_or_create_study_program(pool: &PgPool, name: &str, link: &str) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO study_program (name, link) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET link = $2 RETURNING id",
        name,
        link
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

async fn get_or_create_stupo(pool: &PgPool, study_program_id: i32, name: &str, link: &str) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO stupo (study_program_id, name, link) VALUES ($1, $2, $3) ON CONFLICT (study_program_id, name) DO UPDATE SET link = $3 RETURNING id",
        study_program_id,
        name,
        link
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

fn map_exam(module_id: i32, module_version: i32, scraping_run_id: i32, scraped_exam: ScrapedExam) -> Result<(ExamData, Vec<ExamComponentData>)> {
    let exam_data = ExamData {
        module_id,
        module_version,
        module_scraping_run_id: scraping_run_id,
        graded: scraped_exam.graded,
        exam_type: scraped_exam.exam_type,
        clef: scraped_exam.clef,
        description: scraped_exam.description,
    };

    // Map exam components
    let mut exam_components = Vec::new();
    for component in scraped_exam.components {
        // Map category
        let category = match component.category.as_deref() {
            Some("oral") | Some("Oral") | Some("mündlich") | Some("Mündlich") => db::ExamCategory::Oral,
            Some("written") | Some("Written") | Some("schriftlich") | Some("Schriftlich") => db::ExamCategory::Written,
            Some("praktisch") | Some("Praktisch") | Some("practical") | Some("Practical") => db::ExamCategory::Praktisch,
            _ => db::ExamCategory::Written, // Default to written
        };

        exam_components.push(ExamComponentData {
            name: component.name,
            points: component.points.unwrap_or(0),
            category,
            scope: component.scope,
        });
    }

    Ok((exam_data, exam_components))
}
