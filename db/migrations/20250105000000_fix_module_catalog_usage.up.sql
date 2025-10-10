-- Fix module_catalog_usage table to include module reference and prevent duplicates

-- Add module_id and module_version columns
ALTER TABLE module_catalog_usage
ADD COLUMN module_id INTEGER,
ADD COLUMN module_version INTEGER;

-- Delete all existing data (it's incomplete without module references)
DELETE FROM module_catalog_usage;

-- Make the new columns NOT NULL now that table is empty
ALTER TABLE module_catalog_usage
ALTER COLUMN module_id SET NOT NULL,
ALTER COLUMN module_version SET NOT NULL;

-- Add foreign key constraint to module
ALTER TABLE module_catalog_usage
ADD CONSTRAINT fk_module_catalog_usage_module
    FOREIGN KEY (module_id, module_version)
    REFERENCES module(id, version) ON DELETE CASCADE;

-- Add unique constraint to prevent duplicates
ALTER TABLE module_catalog_usage
ADD CONSTRAINT unique_module_stupo_usage
    UNIQUE (module_id, module_version, stupo_id);
