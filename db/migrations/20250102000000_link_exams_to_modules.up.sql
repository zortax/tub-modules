-- Migration to link exams to modules

ALTER TABLE exam
ADD COLUMN module_id INTEGER NOT NULL,
ADD COLUMN module_version INTEGER NOT NULL,
ADD CONSTRAINT fk_exam_module FOREIGN KEY (module_id, module_version)
    REFERENCES module(id, version) ON DELETE CASCADE;

-- Add index for better query performance
CREATE INDEX idx_exam_module ON exam(module_id, module_version);
