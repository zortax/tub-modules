use crate::models::*;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use db::PgPool;

/// Get complete module details including all related information
#[server(GetModuleDetail)]
#[cfg_attr(feature = "ssr", tracing::instrument(level = "info", fields(module_id = id, module_version = version)))]
pub async fn get_module_detail(id: i32, version: i32) -> Result<ModuleDetail, ServerFnError> {
    use leptos_actix::extract;
    use sqlx::query;

    #[cfg(feature = "ssr")]
    tracing::info!("get_module_detail called for module_id={}, version={}", id, version);

    let pool = extract::<actix_web::web::Data<PgPool>>().await?;
    let pool: &PgPool = &*pool;

    // Fetch main module data with joined organizations
    let module_row = query!(
        r#"
        SELECT
            m.id, m.version, m.title, m.credits, m.languages,
            m.valid_since_semester::text as valid_since_semester,
            m.valid_since_year,
            m.valid_until_semester::text as valid_until_semester,
            m.valid_until_year,
            f.name as faculty_name,
            i.name as institute_name,
            fg.name as fachgebiet_name,
            rp.name as responsible_person_name,
            eb.name as examination_board_name,
            m.learning_result,
            m.content,
            m.teaching_information,
            m.requirements,
            m.additional_info,
            m.registration,
            m.max_attendees,
            m.duration,
            m.moses_link
        FROM module m
        LEFT JOIN faculty f ON m.faculty_id = f.id
        LEFT JOIN institute i ON m.institute_id = i.id
        LEFT JOIN fachgebiet fg ON m.fg_id = fg.id
        LEFT JOIN responsible_person rp ON m.responsible_id = rp.id
        LEFT JOIN examination_board eb ON m.examination_board_id = eb.id
        WHERE m.id = $1 AND m.version = $2
        "#,
        id,
        version
    )
    .fetch_one(pool)
    .await?;

    // Format validity period
    let valid_since = match (
        module_row.valid_since_semester.as_ref(),
        module_row.valid_since_year,
    ) {
        (Some(sem), Some(year)) => Some(format!("{} {}", sem, year)),
        _ => None,
    };

    let valid_until = match (
        module_row.valid_until_semester.as_ref(),
        module_row.valid_until_year,
    ) {
        (Some(sem), Some(year)) => Some(format!("{} {}", sem, year)),
        _ => None,
    };

    // Fetch contact information
    let contact = query!(
        r#"
        SELECT secretariat, contact_person, email, website
        FROM contact
        WHERE module_id = $1 AND module_version = $2
        "#,
        id,
        version
    )
    .fetch_optional(pool)
    .await?
    .map(|row| ContactInfo {
        secretariat: row.secretariat,
        contact_person: row.contact_person,
        email: row.email,
        website: row.website,
    });

    // Fetch module components
    let components = query!(
        r#"
        SELECT
            component_type::text as "component_type!",
            module_name,
            number,
            rotation::text as "rotation!",
            sws,
            language
        FROM module_component
        WHERE module_id = $1 AND module_version = $2
        ORDER BY component_type, number
        "#,
        id,
        version
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|c| ComponentInfo {
        component_type: c.component_type,
        name: c.module_name,
        number: c.number,
        rotation: c.rotation,
        sws: c.sws,
        language: c.language,
    })
    .collect();

    // Fetch exams with components
    let exam_rows = query!(
        r#"
        SELECT id, graded, exam_type, description
        FROM exam
        WHERE module_id = $1 AND module_version = $2
        "#,
        id,
        version
    )
    .fetch_all(pool)
    .await?;

    let mut exams = Vec::new();
    for exam_row in exam_rows {
        let exam_components = query!(
            r#"
            SELECT
                name,
                points,
                category::text as "category!",
                scope
            FROM exam_component
            WHERE exam_id = $1
            "#,
            exam_row.id
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|ec| ExamComponentInfo {
            name: ec.name,
            points: ec.points,
            category: ec.category,
            scope: ec.scope,
        })
        .collect();

        exams.push(ExamInfo {
            id: exam_row.id,
            graded: exam_row.graded,
            exam_type: exam_row.exam_type,
            description: exam_row.description,
            components: exam_components,
        });
    }

    // Fetch workload distribution
    let workload = query!(
        r#"
        SELECT description, total_hours as hours
        FROM module_workload_distribution
        WHERE module_id = $1 AND module_version = $2
        ORDER BY id
        "#,
        id,
        version
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|w| WorkloadInfo {
        description: w.description,
        hours: w.hours,
    })
    .collect();

    // Fetch study programs
    let study_programs = query!(
        r#"
        SELECT
            sp.name as program_name,
            st.name as stupo_name,
            mcu.first_usage,
            mcu.last_usage
        FROM module_catalog_usage mcu
        JOIN stupo st ON mcu.stupo_id = st.id
        JOIN study_program sp ON st.study_program_id = sp.id
        WHERE mcu.module_id = $1 AND mcu.module_version = $2
        ORDER BY sp.name, st.name
        "#,
        id,
        version
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|sp| StudyProgramInfo {
        program_name: sp.program_name,
        stupo_name: sp.stupo_name,
        first_usage: sp.first_usage,
        last_usage: sp.last_usage,
    })
    .collect();

    Ok(ModuleDetail {
        id: module_row.id,
        version: module_row.version,
        title: module_row.title,
        credits: module_row.credits,
        languages: module_row.languages,
        valid_since,
        valid_until,
        faculty: module_row.faculty_name,
        institute: module_row.institute_name,
        fachgebiet: module_row.fachgebiet_name,
        responsible_person: module_row.responsible_person_name,
        examination_board: module_row.examination_board_name,
        contact,
        learning_result: module_row.learning_result,
        content: module_row.content,
        teaching_information: module_row.teaching_information,
        requirements: module_row.requirements,
        additional_info: module_row.additional_info,
        registration: module_row.registration,
        max_attendees: module_row.max_attendees,
        duration: module_row.duration,
        components,
        exams,
        workload,
        study_programs,
        moses_link: module_row.moses_link,
    })
}
