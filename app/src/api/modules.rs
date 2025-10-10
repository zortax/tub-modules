use crate::models::{FilterOptions, ModuleSummary, SearchFilters};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use db::PgPool;

#[cfg(feature = "ssr")]
use crate::models::ComponentInfo;

/// Get total count of modules matching filters
#[server(GetModuleCount)]
pub async fn get_module_count(
    #[server(default)] filters: SearchFilters,
) -> Result<i64, ServerFnError> {
    use leptos_actix::extract;
    use sqlx::Row;

    let pool = extract::<actix_web::web::Data<PgPool>>()
        .await
        ?;
    let pool: &PgPool = &*pool;

    // Build dynamic WHERE clauses
    let mut where_clauses = vec!["1=1".to_string()];

    if let Some(ref query) = filters.search_query {
        if !query.is_empty() {
            where_clauses.push(format!(
                "m.title ILIKE '%{}%'",
                query.replace('\'', "''")
            ));
        }
    }

    if let Some(min) = filters.min_credits {
        where_clauses.push(format!("m.credits >= {}", min));
    }
    if let Some(max) = filters.max_credits {
        where_clauses.push(format!("m.credits <= {}", max));
    }

    if let Some(ref rotations) = filters.semester_rotations {
        if !rotations.is_empty() {
            let rotation_strs: Vec<String> = rotations
                .iter()
                .map(|r| format!("'{}'", r.replace('\'', "''")))
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_component mc WHERE mc.module_id = m.id AND mc.module_version = m.version AND mc.rotation::text IN ({}))",
                rotation_strs.join(", ")
            ));
        }
    }

    if let Some(ref categories) = filters.exam_categories {
        if !categories.is_empty() {
            let category_strs: Vec<String> = categories
                .iter()
                .map(|c| format!("'{}'", c.replace('\'', "''")))
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM exam e JOIN exam_component ec ON e.id = ec.exam_id WHERE e.module_id = m.id AND e.module_version = m.version AND ec.category::text IN ({}))",
                category_strs.join(", ")
            ));
        }
    }

    if let Some(ref ids) = filters.study_program_ids {
        if !ids.is_empty() {
            let id_strs: Vec<String> = ids
                .iter()
                .map(|id| id.to_string())
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_catalog_usage mcu JOIN stupo st ON mcu.stupo_id = st.id WHERE mcu.module_id = m.id AND mcu.module_version = m.version AND st.study_program_id IN ({}))",
                id_strs.join(", ")
            ));
        }
    }

    if let Some(ref types) = filters.component_types {
        if !types.is_empty() {
            let type_strs: Vec<String> = types
                .iter()
                .map(|t| format!("'{}'", t.replace('\'', "''")))
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_component mc WHERE mc.module_id = m.id AND mc.module_version = m.version AND mc.component_type::text IN ({}))",
                type_strs.join(", ")
            ));
        }
    }

    if let Some(ref langs) = filters.component_languages {
        if !langs.is_empty() {
            let lang_conditions: Vec<String> = langs
                .iter()
                .map(|l| {
                    let escaped = l.replace('\'', "''");
                    format!("mc.language ~ '(^|, ){}(, |$)'", escaped)
                })
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_component mc WHERE mc.module_id = m.id AND mc.module_version = m.version AND ({}))",
                lang_conditions.join(" OR ")
            ));
        }
    }

    let where_clause = where_clauses.join(" AND ");

    let query_str = format!(
        r#"
        SELECT COUNT(DISTINCT (m.id, m.version))
        FROM module m
        WHERE {}
        "#,
        where_clause
    );

    let row = sqlx::query(&query_str)
        .fetch_one(pool)
        .await
        ?;

    let count: i64 = row.try_get(0).unwrap_or(0);

    Ok(count)
}

/// Get filter options from the database (study programs, exam categories, etc.)
#[server(GetFilterOptions)]
pub async fn get_filter_options() -> Result<FilterOptions, ServerFnError> {
    use leptos_actix::extract;
    use sqlx::query;

    let pool = extract::<actix_web::web::Data<PgPool>>()
        .await
        ?;
    let pool: &PgPool = &*pool;

    // Get all study programs
    let study_programs = query!(
        r#"
        SELECT id, name
        FROM study_program
        ORDER BY name
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| crate::models::StudyProgramOption {
        id: row.id,
        name: row.name,
    })
    .collect();

    // Get distinct exam categories
    let exam_categories = query!(
        r#"
        SELECT DISTINCT category::text as "category!"
        FROM exam_component
        ORDER BY category::text
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.category)
    .collect();

    // Get distinct semester rotations
    let semester_rotations = query!(
        r#"
        SELECT DISTINCT rotation::text as "rotation!"
        FROM module_component
        ORDER BY rotation::text
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.rotation)
    .collect();

    // Get distinct component types
    let component_types = query!(
        r#"
        SELECT DISTINCT component_type::text as "component_type!"
        FROM module_component
        ORDER BY component_type::text
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.component_type)
    .collect();

    // Get distinct component languages
    let component_languages = query!(
        r#"
        SELECT DISTINCT language as "language!"
        FROM module_component
        WHERE language IS NOT NULL AND language != ''
        ORDER BY language
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.language)
    .collect();

    // Get credit range
    let credit_range = query!(
        r#"
        SELECT MIN(credits) as "min!", MAX(credits) as "max!"
        FROM module
        "#
    )
    .fetch_one(pool)
    .await
    ?;

    Ok(FilterOptions {
        study_programs,
        exam_categories,
        semester_rotations,
        component_types,
        component_languages,
        credit_range: (credit_range.min, credit_range.max),
    })
}

/// Search modules with filters and pagination
#[server(SearchModulesPaginated)]
pub async fn search_modules_paginated(
    #[server(default)] filters: SearchFilters,
    page: i64,
    page_size: i64,
) -> Result<Vec<ModuleSummary>, ServerFnError> {
    use leptos_actix::extract;
    use sqlx::{query, Row};

    let pool = extract::<actix_web::web::Data<PgPool>>()
        .await
        ?;
    let pool: &PgPool = &*pool;

    // Build dynamic WHERE clauses
    let mut where_clauses = vec!["1=1".to_string()];

    // Search query filter
    if let Some(ref query) = filters.search_query {
        if !query.is_empty() {
            where_clauses.push(format!(
                "m.title ILIKE '%{}%'",
                query.replace('\'', "''")
            ));
        }
    }

    // Credit filters
    if let Some(min) = filters.min_credits {
        where_clauses.push(format!("m.credits >= {}", min));
    }
    if let Some(max) = filters.max_credits {
        where_clauses.push(format!("m.credits <= {}", max));
    }

    // Semester rotation filter
    if let Some(ref rotations) = filters.semester_rotations {
        if !rotations.is_empty() {
            let rotation_strs: Vec<String> = rotations
                .iter()
                .map(|r| format!("'{}'", r.replace('\'', "''")))
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_component mc WHERE mc.module_id = m.id AND mc.module_version = m.version AND mc.rotation::text IN ({}))",
                rotation_strs.join(", ")
            ));
        }
    }

    // Exam category filter
    if let Some(ref categories) = filters.exam_categories {
        if !categories.is_empty() {
            let category_strs: Vec<String> = categories
                .iter()
                .map(|c| format!("'{}'", c.replace('\'', "''")))
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM exam e JOIN exam_component ec ON e.id = ec.exam_id WHERE e.module_id = m.id AND e.module_version = m.version AND ec.category::text IN ({}))",
                category_strs.join(", ")
            ));
        }
    }

    // Study program filter
    if let Some(ref ids) = filters.study_program_ids {
        if !ids.is_empty() {
            let id_strs: Vec<String> = ids
                .iter()
                .map(|id| id.to_string())
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_catalog_usage mcu JOIN stupo st ON mcu.stupo_id = st.id WHERE mcu.module_id = m.id AND mcu.module_version = m.version AND st.study_program_id IN ({}))",
                id_strs.join(", ")
            ));
        }
    }

    // Component type filter
    if let Some(ref types) = filters.component_types {
        if !types.is_empty() {
            let type_strs: Vec<String> = types
                .iter()
                .map(|t| format!("'{}'", t.replace('\'', "''")))
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_component mc WHERE mc.module_id = m.id AND mc.module_version = m.version AND mc.component_type::text IN ({}))",
                type_strs.join(", ")
            ));
        }
    }

    // Component language filter
    if let Some(ref langs) = filters.component_languages {
        if !langs.is_empty() {
            let lang_conditions: Vec<String> = langs
                .iter()
                .map(|l| {
                    let escaped = l.replace('\'', "''");
                    format!("mc.language ~ '(^|, ){}(, |$)'", escaped)
                })
                .collect();
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM module_component mc WHERE mc.module_id = m.id AND mc.module_version = m.version AND ({}))",
                lang_conditions.join(" OR ")
            ));
        }
    }

    let where_clause = where_clauses.join(" AND ");
    let offset = page * page_size;

    // Main query
    let query_str = format!(
        r#"
        SELECT DISTINCT
            m.id,
            m.version,
            m.title,
            m.credits,
            m.languages,
            f.name as faculty_name
        FROM module m
        LEFT JOIN faculty f ON m.faculty_id = f.id
        WHERE {}
        ORDER BY m.title
        LIMIT {} OFFSET {}
        "#,
        where_clause, page_size, offset
    );

    let rows = sqlx::query(&query_str)
        .fetch_all(pool)
        .await
        ?;

    let mut modules_data = Vec::new();

    for row in rows {
        let id: i32 = row.try_get("id").unwrap_or(0);
        let version: i32 = row.try_get("version").unwrap_or(0);

        // Get semester rotations
        let rotations = query!(
            r#"
            SELECT DISTINCT rotation::text as "rotation!"
            FROM module_component
            WHERE module_id = $1 AND module_version = $2
            "#,
            id,
            version
        )
        .fetch_all(pool)
        .await
        ?
        .into_iter()
        .map(|r| r.rotation)
        .collect();

        // Get exam categories
        let categories = query!(
            r#"
            SELECT DISTINCT ec.category::text as "category!"
            FROM exam e
            JOIN exam_component ec ON e.id = ec.exam_id
            WHERE e.module_id = $1 AND e.module_version = $2
            "#,
            id,
            version
        )
        .fetch_all(pool)
        .await
        ?
        .into_iter()
        .map(|r| r.category)
        .collect();

        // Get study programs
        let programs = query!(
            r#"
            SELECT DISTINCT sp.id, sp.name
            FROM module_catalog_usage mcu
            JOIN stupo st ON mcu.stupo_id = st.id
            JOIN study_program sp ON st.study_program_id = sp.id
            WHERE mcu.module_id = $1 AND mcu.module_version = $2
            "#,
            id,
            version
        )
        .fetch_all(pool)
        .await
        ?;

        let study_program_names: Vec<String> = programs.iter().map(|p| p.name.clone()).collect();
        let study_program_ids: Vec<i32> = programs.iter().map(|p| p.id).collect();

        // Get module components
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
        .await
        ?
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

        modules_data.push(ModuleSummary {
            id,
            version,
            title: row.try_get("title").unwrap_or_default(),
            credits: row.try_get("credits").unwrap_or(0),
            languages: row.try_get("languages").unwrap_or_default(),
            semester_rotations: rotations,
            exam_categories: categories,
            study_programs: study_program_names,
            study_program_ids,
            faculty_name: row.try_get("faculty_name").unwrap_or_default(),
            components,
        });
    }

    Ok(modules_data)
}
