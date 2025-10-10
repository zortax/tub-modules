use anyhow::{Context, Result};
use sqlx::PgPool;

use crate::mapper::MappedModuleData;

pub async fn insert_module_data(pool: &PgPool, data: MappedModuleData) -> Result<()> {
    // Use a transaction to ensure atomicity
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        INSERT INTO module (
            id, version, valid_since_semester, valid_since_year,
            valid_until_semester, valid_until_year, languages, title,
            credits, m_pord_nr, m_p_nr, mp_pord_nr, mp_p_nr,
            faculty_id, institute_id, fg_id, responsible_id, examination_board_id,
            learning_result, content, teaching_information, max_attendees,
            registration, duration, requirements, additional_info, moses_link
        )
        VALUES ($1, $2, $3::semester, $4, $5::semester, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)
        ON CONFLICT (id, version) DO UPDATE SET
            valid_since_semester = $3::semester,
            valid_since_year = $4,
            valid_until_semester = $5::semester,
            valid_until_year = $6,
            languages = $7,
            title = $8,
            credits = $9,
            m_pord_nr = $10,
            m_p_nr = $11,
            mp_pord_nr = $12,
            mp_p_nr = $13,
            faculty_id = $14,
            institute_id = $15,
            fg_id = $16,
            responsible_id = $17,
            examination_board_id = $18,
            learning_result = $19,
            content = $20,
            teaching_information = $21,
            max_attendees = $22,
            registration = $23,
            duration = $24,
            requirements = $25,
            additional_info = $26,
            moses_link = $27
        "#,
        data.module.id,
        data.module.version,
        data.module.valid_since_semester as Option<db::Semester>,
        data.module.valid_since_year,
        data.module.valid_until_semester as Option<db::Semester>,
        data.module.valid_until_year,
        &data.module.languages,
        data.module.title,
        data.module.credits,
        data.module.m_pord_nr,
        data.module.m_p_nr,
        data.module.mp_pord_nr,
        data.module.mp_p_nr,
        data.module.faculty_id,
        data.module.institute_id,
        data.module.fg_id,
        data.module.responsible_id,
        data.module.examination_board_id,
        data.module.learning_result,
        data.module.content,
        data.module.teaching_information,
        data.module.max_attendees,
        data.module.registration,
        data.module.duration,
        data.module.requirements,
        data.module.additional_info,
        data.module.moses_link
    )
    .execute(&mut *tx)
    .await
    .context("Failed to insert module")?;

    // Delete existing contact if updating
    sqlx::query!(
        "DELETE FROM contact WHERE module_id = $1 AND module_version = $2",
        data.module.id,
        data.module.version
    )
    .execute(&mut *tx)
    .await?;

    // Insert contact if present
    if let Some(contact) = data.contact {
        sqlx::query!(
            r#"
            INSERT INTO contact (module_id, module_version, secretariat, contact_person, email, website)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            contact.module_id,
            contact.module_version,
            contact.secretariat,
            contact.contact_person,
            contact.email,
            contact.website
        )
        .execute(&mut *tx)
        .await
        .context("Failed to insert contact")?;
    }

    // Delete existing components if updating
    sqlx::query!(
        "DELETE FROM module_component WHERE module_id = $1 AND module_version = $2",
        data.module.id,
        data.module.version
    )
    .execute(&mut *tx)
    .await?;

    // Insert components
    for component in data.components {
        sqlx::query!(
            r#"
            INSERT INTO module_component (
                module_id, module_version, module_name, component_type,
                number, rotation, sws, language
            )
            VALUES ($1, $2, $3, $4, $5, $6::component_rotation, $7, $8)
            "#,
            component.module_id,
            component.module_version,
            component.module_name,
            component.component_type,
            component.number,
            component.rotation as db::ComponentRotation,
            component.sws,
            component.language
        )
        .execute(&mut *tx)
        .await
        .context("Failed to insert module component")?;
    }

    // Delete existing workload if updating
    sqlx::query!(
        "DELETE FROM module_workload_distribution WHERE module_id = $1 AND module_version = $2",
        data.module.id,
        data.module.version
    )
    .execute(&mut *tx)
    .await?;

    // Insert workload
    for workload in data.workload {
        sqlx::query!(
            r#"
            INSERT INTO module_workload_distribution (
                module_id, module_version, description, factor, hours, total_hours
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            workload.module_id,
            workload.module_version,
            workload.description,
            workload.factor,
            workload.hours,
            workload.total_hours
        )
        .execute(&mut *tx)
        .await
        .context("Failed to insert workload distribution")?;
    }

    // Insert study program usages
    for usage in data.study_program_usages {
        sqlx::query!(
            r#"
            INSERT INTO module_catalog_usage (module_id, module_version, stupo_id, first_usage, last_usage)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (module_id, module_version, stupo_id) DO UPDATE SET
                first_usage = $4,
                last_usage = $5
            "#,
            usage.module_id,
            usage.module_version,
            usage.stupo_id,
            usage.first_usage,
            usage.last_usage
        )
        .execute(&mut *tx)
        .await
        .context("Failed to insert module catalog usage")?;
    }

    // Delete existing exam if updating
    sqlx::query!(
        "DELETE FROM exam WHERE module_id = $1 AND module_version = $2",
        data.module.id,
        data.module.version
    )
    .execute(&mut *tx)
    .await?;

    // Insert exam if present
    if let Some(exam) = data.exam {
        let exam_id = sqlx::query!(
            r#"
            INSERT INTO exam (module_id, module_version, graded, exam_type, clef, description)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            exam.module_id,
            exam.module_version,
            exam.graded,
            exam.exam_type,
            exam.clef,
            exam.description
        )
        .fetch_one(&mut *tx)
        .await
        .context("Failed to insert exam")?
        .id;

        // Insert exam components
        for component in data.exam_components {
            sqlx::query!(
                r#"
                INSERT INTO exam_component (exam_id, name, points, category, scope)
                VALUES ($1, $2, $3, $4::exam_category, $5)
                "#,
                exam_id,
                component.name,
                component.points,
                component.category as db::ExamCategory,
                component.scope
            )
            .execute(&mut *tx)
            .await
            .context("Failed to insert exam component")?;
        }
    }

    // Commit transaction
    tx.commit().await?;

    Ok(())
}
