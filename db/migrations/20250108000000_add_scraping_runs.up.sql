-- Add scraping run tracking to preserve historical data
-- This migration allows multiple snapshots of the same module to exist

-- ============================================================================
-- Step 1: Create scraping_run table
-- ============================================================================

CREATE TABLE scraping_run (
    id SERIAL PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'in_progress',
    total_modules INTEGER,
    successful_modules INTEGER DEFAULT 0,
    failed_modules INTEGER DEFAULT 0,
    skipped_modules INTEGER DEFAULT 0
);

CREATE INDEX idx_scraping_run_status ON scraping_run(status);
CREATE INDEX idx_scraping_run_completed_at ON scraping_run(completed_at DESC);

-- ============================================================================
-- Step 2: Create initial scraping run for existing data
-- ============================================================================

-- Insert a scraping run for all existing data
INSERT INTO scraping_run (started_at, completed_at, status, successful_modules)
VALUES (NOW(), NOW(), 'completed', (SELECT COUNT(*) FROM module));

-- Store the run_id for use in following steps
DO $$
DECLARE
    initial_run_id INTEGER;
BEGIN
    SELECT id INTO initial_run_id FROM scraping_run ORDER BY id DESC LIMIT 1;

    -- ============================================================================
    -- Step 3: Add scraping_run_id to module table
    -- ============================================================================

    -- Add the column (nullable initially)
    ALTER TABLE module ADD COLUMN scraping_run_id INTEGER;

    -- Set all existing modules to the initial run
    EXECUTE format('UPDATE module SET scraping_run_id = %s', initial_run_id);

    -- Make it NOT NULL now
    ALTER TABLE module ALTER COLUMN scraping_run_id SET NOT NULL;

    -- Add foreign key constraint
    ALTER TABLE module
    ADD CONSTRAINT fk_module_scraping_run
        FOREIGN KEY (scraping_run_id)
        REFERENCES scraping_run(id) ON DELETE RESTRICT;

    -- ============================================================================
    -- Step 4: Drop all child table foreign keys that reference module
    -- ============================================================================

    ALTER TABLE contact DROP CONSTRAINT fk_contact_module;
    ALTER TABLE module_component DROP CONSTRAINT fk_module_component_module;
    ALTER TABLE module_workload_distribution DROP CONSTRAINT fk_module_workload_module;
    ALTER TABLE exam DROP CONSTRAINT fk_exam_module;
    ALTER TABLE module_catalog_usage DROP CONSTRAINT fk_module_catalog_usage_module;

    -- ============================================================================
    -- Step 5: Update module primary key
    -- ============================================================================

    -- Drop the old primary key and create new one
    ALTER TABLE module DROP CONSTRAINT module_pkey;
    ALTER TABLE module ADD PRIMARY KEY (id, version, scraping_run_id);

    -- Add index for performance
    CREATE INDEX idx_module_scraping_run ON module(scraping_run_id);

    -- ============================================================================
    -- Step 6: Update child tables and recreate foreign keys
    -- ============================================================================

    -- CONTACT table
    ALTER TABLE contact ADD COLUMN module_scraping_run_id INTEGER;

    UPDATE contact c
    SET module_scraping_run_id = m.scraping_run_id
    FROM module m
    WHERE c.module_id = m.id AND c.module_version = m.version;

    ALTER TABLE contact ALTER COLUMN module_scraping_run_id SET NOT NULL;

    ALTER TABLE contact
    ADD CONSTRAINT fk_contact_module
        FOREIGN KEY (module_id, module_version, module_scraping_run_id)
        REFERENCES module(id, version, scraping_run_id) ON DELETE CASCADE;

    CREATE INDEX idx_contact_scraping_run ON contact(module_scraping_run_id);

    -- MODULE_COMPONENT table
    ALTER TABLE module_component ADD COLUMN module_scraping_run_id INTEGER;

    UPDATE module_component mc
    SET module_scraping_run_id = m.scraping_run_id
    FROM module m
    WHERE mc.module_id = m.id AND mc.module_version = m.version;

    ALTER TABLE module_component ALTER COLUMN module_scraping_run_id SET NOT NULL;

    ALTER TABLE module_component
    ADD CONSTRAINT fk_module_component_module
        FOREIGN KEY (module_id, module_version, module_scraping_run_id)
        REFERENCES module(id, version, scraping_run_id) ON DELETE CASCADE;

    CREATE INDEX idx_module_component_scraping_run ON module_component(module_scraping_run_id);

    -- MODULE_WORKLOAD_DISTRIBUTION table
    ALTER TABLE module_workload_distribution ADD COLUMN module_scraping_run_id INTEGER;

    UPDATE module_workload_distribution mwd
    SET module_scraping_run_id = m.scraping_run_id
    FROM module m
    WHERE mwd.module_id = m.id AND mwd.module_version = m.version;

    ALTER TABLE module_workload_distribution ALTER COLUMN module_scraping_run_id SET NOT NULL;

    ALTER TABLE module_workload_distribution
    ADD CONSTRAINT fk_module_workload_module
        FOREIGN KEY (module_id, module_version, module_scraping_run_id)
        REFERENCES module(id, version, scraping_run_id) ON DELETE CASCADE;

    -- EXAM table
    ALTER TABLE exam ADD COLUMN module_scraping_run_id INTEGER;

    UPDATE exam e
    SET module_scraping_run_id = m.scraping_run_id
    FROM module m
    WHERE e.module_id = m.id AND e.module_version = m.version;

    ALTER TABLE exam ALTER COLUMN module_scraping_run_id SET NOT NULL;

    ALTER TABLE exam
    ADD CONSTRAINT fk_exam_module
        FOREIGN KEY (module_id, module_version, module_scraping_run_id)
        REFERENCES module(id, version, scraping_run_id) ON DELETE CASCADE;

    CREATE INDEX idx_exam_scraping_run ON exam(module_scraping_run_id);

    -- MODULE_CATALOG_USAGE table
    ALTER TABLE module_catalog_usage ADD COLUMN module_scraping_run_id INTEGER;

    UPDATE module_catalog_usage mcu
    SET module_scraping_run_id = m.scraping_run_id
    FROM module m
    WHERE mcu.module_id = m.id AND mcu.module_version = m.version;

    ALTER TABLE module_catalog_usage ALTER COLUMN module_scraping_run_id SET NOT NULL;

    ALTER TABLE module_catalog_usage DROP CONSTRAINT unique_module_stupo_usage;
    ALTER TABLE module_catalog_usage
    ADD CONSTRAINT fk_module_catalog_usage_module
        FOREIGN KEY (module_id, module_version, module_scraping_run_id)
        REFERENCES module(id, version, scraping_run_id) ON DELETE CASCADE;
    ALTER TABLE module_catalog_usage
    ADD CONSTRAINT unique_module_stupo_usage
        UNIQUE (module_id, module_version, module_scraping_run_id, stupo_id);
END $$;
