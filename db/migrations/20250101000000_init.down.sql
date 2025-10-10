-- Initial migration (down)
-- Rollback schema for TU Berlin module scraper

-- ============================================================================
-- Drop Tables in Reverse Dependency Order
-- ============================================================================

-- Drop tables that depend on stupo
DROP TABLE IF EXISTS module_catalog_usage;

-- Drop tables that depend on study_program
DROP TABLE IF EXISTS stupo;

-- Drop tables that depend on module
DROP TABLE IF EXISTS module_workload_distribution;
DROP TABLE IF EXISTS module_component;
DROP TABLE IF EXISTS contact;

-- Drop tables that depend on exam
DROP TABLE IF EXISTS exam_component;

-- Drop tables with no dependencies
DROP TABLE IF EXISTS exam;
DROP TABLE IF EXISTS module;
DROP TABLE IF EXISTS study_program;
DROP TABLE IF EXISTS responsible_person;
DROP TABLE IF EXISTS examination_board;
DROP TABLE IF EXISTS fachgebiet;
DROP TABLE IF EXISTS institute;
DROP TABLE IF EXISTS faculty;

-- ============================================================================
-- Drop ENUM Types
-- ============================================================================

DROP TYPE IF EXISTS component_rotation;
DROP TYPE IF EXISTS component_type;
DROP TYPE IF EXISTS exam_category;
DROP TYPE IF EXISTS exam_type;
DROP TYPE IF EXISTS semester;
