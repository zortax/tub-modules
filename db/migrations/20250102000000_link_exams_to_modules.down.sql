-- Rollback exam-module link

DROP INDEX IF EXISTS idx_exam_module;

ALTER TABLE exam
DROP CONSTRAINT IF EXISTS fk_exam_module,
DROP COLUMN IF EXISTS module_version,
DROP COLUMN IF EXISTS module_id;
