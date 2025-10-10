-- Change exam_type from ENUM to TEXT for flexibility

-- First, alter the exam table to use TEXT
ALTER TABLE exam
ALTER COLUMN exam_type TYPE TEXT USING exam_type::TEXT;

-- Drop the old enum type (this will only work if no other tables use it)
DROP TYPE IF EXISTS exam_type;
