-- Rollback module_catalog_usage table changes

-- Drop unique constraint
ALTER TABLE module_catalog_usage
DROP CONSTRAINT IF EXISTS unique_module_stupo_usage;

-- Drop foreign key constraint
ALTER TABLE module_catalog_usage
DROP CONSTRAINT IF EXISTS fk_module_catalog_usage_module;

-- Drop the module reference columns
ALTER TABLE module_catalog_usage
DROP COLUMN IF EXISTS module_version,
DROP COLUMN IF EXISTS module_id;
