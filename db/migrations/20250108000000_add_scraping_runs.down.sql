-- Rollback scraping run tracking migration

-- ============================================================================
-- Step 1: Restore module primary key to (id, version)
-- ============================================================================

ALTER TABLE module DROP CONSTRAINT module_pkey;
ALTER TABLE module ADD PRIMARY KEY (id, version);

-- ============================================================================
-- Step 2: Drop indexes on scraping_run_id
-- ============================================================================

DROP INDEX IF EXISTS idx_module_scraping_run;
DROP INDEX IF EXISTS idx_contact_scraping_run;
DROP INDEX IF EXISTS idx_module_component_scraping_run;
DROP INDEX IF EXISTS idx_exam_scraping_run;

-- ============================================================================
-- Step 3: Restore child table foreign keys (two-column)
-- ============================================================================

-- MODULE_CATALOG_USAGE
ALTER TABLE module_catalog_usage DROP CONSTRAINT fk_module_catalog_usage_module;
ALTER TABLE module_catalog_usage DROP CONSTRAINT unique_module_stupo_usage;
ALTER TABLE module_catalog_usage DROP COLUMN module_scraping_run_id;
ALTER TABLE module_catalog_usage
ADD CONSTRAINT fk_module_catalog_usage_module
    FOREIGN KEY (module_id, module_version)
    REFERENCES module(id, version) ON DELETE CASCADE;
ALTER TABLE module_catalog_usage
ADD CONSTRAINT unique_module_stupo_usage
    UNIQUE (module_id, module_version, stupo_id);

-- EXAM
ALTER TABLE exam DROP CONSTRAINT fk_exam_module;
ALTER TABLE exam DROP COLUMN module_scraping_run_id;
ALTER TABLE exam
ADD CONSTRAINT fk_exam_module
    FOREIGN KEY (module_id, module_version)
    REFERENCES module(id, version) ON DELETE CASCADE;

-- MODULE_WORKLOAD_DISTRIBUTION
ALTER TABLE module_workload_distribution DROP CONSTRAINT fk_module_workload_module;
ALTER TABLE module_workload_distribution DROP COLUMN module_scraping_run_id;
ALTER TABLE module_workload_distribution
ADD CONSTRAINT fk_module_workload_module
    FOREIGN KEY (module_id, module_version)
    REFERENCES module(id, version) ON DELETE CASCADE;

-- MODULE_COMPONENT
ALTER TABLE module_component DROP CONSTRAINT fk_module_component_module;
ALTER TABLE module_component DROP COLUMN module_scraping_run_id;
ALTER TABLE module_component
ADD CONSTRAINT fk_module_component_module
    FOREIGN KEY (module_id, module_version)
    REFERENCES module(id, version) ON DELETE CASCADE;

-- CONTACT
ALTER TABLE contact DROP CONSTRAINT fk_contact_module;
ALTER TABLE contact DROP COLUMN module_scraping_run_id;
ALTER TABLE contact
ADD CONSTRAINT fk_contact_module
    FOREIGN KEY (module_id, module_version)
    REFERENCES module(id, version) ON DELETE CASCADE;

-- ============================================================================
-- Step 4: Remove scraping_run_id from module
-- ============================================================================

ALTER TABLE module DROP CONSTRAINT fk_module_scraping_run;
ALTER TABLE module DROP COLUMN scraping_run_id;

-- ============================================================================
-- Step 5: Drop scraping_run table
-- ============================================================================

DROP INDEX IF EXISTS idx_scraping_run_completed_at;
DROP INDEX IF EXISTS idx_scraping_run_status;
DROP TABLE scraping_run;
