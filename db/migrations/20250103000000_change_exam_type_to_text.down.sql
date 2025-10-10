-- Rollback exam_type from TEXT to ENUM

-- Recreate the enum type
CREATE TYPE exam_type AS ENUM ('Portfoliepruefung');

-- Alter the exam table back to use the enum
-- Note: This will fail if there are exam types that don't match 'Portfoliepruefung'
ALTER TABLE exam
ALTER COLUMN exam_type TYPE exam_type USING 'Portfoliepruefung'::exam_type;
