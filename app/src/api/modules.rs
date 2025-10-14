use crate::models::{FilterOptions, ModuleSummary, SearchFilters};
use leptos::prelude::*;
use chrono::{DateTime, Utc};

#[cfg(feature = "ssr")]
use db::PgPool;

#[cfg(feature = "ssr")]
use crate::models::ComponentInfo;

/// Get the timestamp of the latest completed scraping run
#[server(GetLatestScrapingRun)]
#[cfg_attr(feature = "ssr", tracing::instrument(level = "info"))]
pub async fn get_latest_scraping_run() -> Result<Option<DateTime<Utc>>, ServerFnError> {
    use leptos_actix::extract;

    let pool = extract::<actix_web::web::Data<PgPool>>()
        .await
        ?;
    let pool: &PgPool = &*pool;

    let result = sqlx::query!(
        r#"
        SELECT completed_at
        FROM scraping_run
        WHERE status = 'completed'
        ORDER BY completed_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.and_then(|r| r.completed_at))
}

/// Get total count of modules matching filters
#[server(GetModuleCount)]
#[cfg_attr(feature = "ssr", tracing::instrument(level = "info", skip_all, fields(filter_count = filters.search_query.is_some())))]
pub async fn get_module_count(
    #[server(default)] filters: SearchFilters,
) -> Result<i64, ServerFnError> {
    use leptos_actix::extract;
    use sqlx::Row;

    #[cfg(feature = "ssr")]
    tracing::info!("get_module_count called");

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
        WITH latest_modules AS (
            SELECT DISTINCT ON (id, version) id, version, scraping_run_id
            FROM module
            ORDER BY id, version, scraping_run_id DESC
        )
        SELECT COUNT(DISTINCT (m.id, m.version))
        FROM module m
        INNER JOIN latest_modules lm ON m.id = lm.id AND m.version = lm.version AND m.scraping_run_id = lm.scraping_run_id
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
#[cfg_attr(feature = "ssr", tracing::instrument(level = "info"))]
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

    // Get distinct exam categories (from latest runs only)
    let exam_categories = query!(
        r#"
        WITH latest_modules AS (
            SELECT DISTINCT ON (id, version) id, version, scraping_run_id
            FROM module
            ORDER BY id, version, scraping_run_id DESC
        )
        SELECT DISTINCT ec.category::text as "category!"
        FROM exam_component ec
        JOIN exam e ON ec.exam_id = e.id
        JOIN latest_modules lm ON e.module_id = lm.id AND e.module_version = lm.version AND e.module_scraping_run_id = lm.scraping_run_id
        ORDER BY category::text
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.category)
    .collect();

    // Get distinct semester rotations (from latest runs only)
    let semester_rotations = query!(
        r#"
        WITH latest_modules AS (
            SELECT DISTINCT ON (id, version) id, version, scraping_run_id
            FROM module
            ORDER BY id, version, scraping_run_id DESC
        )
        SELECT DISTINCT mc.rotation::text as "rotation!"
        FROM module_component mc
        JOIN latest_modules lm ON mc.module_id = lm.id AND mc.module_version = lm.version AND mc.module_scraping_run_id = lm.scraping_run_id
        ORDER BY rotation::text
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.rotation)
    .collect();

    // Get distinct component types (from latest runs only)
    let component_types = query!(
        r#"
        WITH latest_modules AS (
            SELECT DISTINCT ON (id, version) id, version, scraping_run_id
            FROM module
            ORDER BY id, version, scraping_run_id DESC
        )
        SELECT DISTINCT mc.component_type::text as "component_type!"
        FROM module_component mc
        JOIN latest_modules lm ON mc.module_id = lm.id AND mc.module_version = lm.version AND mc.module_scraping_run_id = lm.scraping_run_id
        ORDER BY component_type::text
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.component_type)
    .collect();

    // Get distinct component languages (from latest runs only)
    let component_languages = query!(
        r#"
        WITH latest_modules AS (
            SELECT DISTINCT ON (id, version) id, version, scraping_run_id
            FROM module
            ORDER BY id, version, scraping_run_id DESC
        )
        SELECT DISTINCT mc.language as "language!"
        FROM module_component mc
        JOIN latest_modules lm ON mc.module_id = lm.id AND mc.module_version = lm.version AND mc.module_scraping_run_id = lm.scraping_run_id
        WHERE mc.language IS NOT NULL AND mc.language != ''
        ORDER BY language
        "#
    )
    .fetch_all(pool)
    .await
    ?
    .into_iter()
    .map(|row| row.language)
    .collect();

    // Get credit range (from latest runs only)
    let credit_range = query!(
        r#"
        WITH latest_modules AS (
            SELECT DISTINCT ON (id, version) id, version, scraping_run_id
            FROM module
            ORDER BY id, version, scraping_run_id DESC
        )
        SELECT MIN(m.credits) as "min!", MAX(m.credits) as "max!"
        FROM module m
        JOIN latest_modules lm ON m.id = lm.id AND m.version = lm.version AND m.scraping_run_id = lm.scraping_run_id
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
#[cfg_attr(feature = "ssr", tracing::instrument(level = "info", skip(filters), fields(page = page, page_size = page_size)))]
pub async fn search_modules_paginated(
    #[server(default)] filters: SearchFilters,
    page: i64,
    page_size: i64,
) -> Result<Vec<ModuleSummary>, ServerFnError> {
    use leptos_actix::extract;
    use sqlx::{query, Row};

    #[cfg(feature = "ssr")]
    tracing::info!("search_modules_paginated called with page={}, page_size={}", page, page_size);

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

    // Main query with CTE to get latest modules
    let query_str = format!(
        r#"
        WITH latest_modules AS (
            SELECT DISTINCT ON (id, version) id, version, scraping_run_id
            FROM module
            ORDER BY id, version, scraping_run_id DESC
        )
        SELECT DISTINCT
            m.id,
            m.version,
            m.title,
            m.credits,
            m.languages,
            f.name as faculty_name
        FROM module m
        INNER JOIN latest_modules lm ON m.id = lm.id AND m.version = lm.version AND m.scraping_run_id = lm.scraping_run_id
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

        // Get the latest scraping_run_id for this module
        let latest_run_id = query!(
            r#"
            SELECT scraping_run_id
            FROM module
            WHERE id = $1 AND version = $2
            ORDER BY scraping_run_id DESC
            LIMIT 1
            "#,
            id,
            version
        )
        .fetch_one(pool)
        .await?
        .scraping_run_id;

        // Get semester rotations
        let rotations = query!(
            r#"
            SELECT DISTINCT rotation::text as "rotation!"
            FROM module_component
            WHERE module_id = $1 AND module_version = $2 AND module_scraping_run_id = $3
            "#,
            id,
            version,
            latest_run_id
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
            WHERE e.module_id = $1 AND e.module_version = $2 AND e.module_scraping_run_id = $3
            "#,
            id,
            version,
            latest_run_id
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
            WHERE mcu.module_id = $1 AND mcu.module_version = $2 AND mcu.module_scraping_run_id = $3
            "#,
            id,
            version,
            latest_run_id
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
            WHERE module_id = $1 AND module_version = $2 AND module_scraping_run_id = $3
            ORDER BY component_type, number
            "#,
            id,
            version,
            latest_run_id
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
