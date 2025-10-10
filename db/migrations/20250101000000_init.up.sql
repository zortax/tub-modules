-- Initial migration (up)
-- Create schema for TU Berlin module scraper

-- ============================================================================
-- Create ENUM Types
-- ============================================================================

CREATE TYPE semester AS ENUM ('SoSe', 'WiSe');

CREATE TYPE exam_type AS ENUM ('Portfoliepruefung');

CREATE TYPE exam_category AS ENUM ('oral', 'written');

CREATE TYPE component_type AS ENUM ('VL', 'UE', 'PJ');

CREATE TYPE component_rotation AS ENUM ('WiSe', 'SoSe', 'WiSe/SoSe');

-- ============================================================================
-- Create Base Tables (no dependencies)
-- ============================================================================

CREATE TABLE faculty (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE institute (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE fachgebiet (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE examination_board (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE study_program (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    link TEXT NOT NULL
);

CREATE TABLE responsible_person (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    fg_id INTEGER,
    CONSTRAINT fk_responsible_person_fg FOREIGN KEY (fg_id)
        REFERENCES fachgebiet(id) ON DELETE SET NULL
);

-- ============================================================================
-- Create Module Table (composite primary key)
-- ============================================================================

CREATE TABLE module (
    id INTEGER NOT NULL,
    version INTEGER NOT NULL,
    valid_since_semester semester,
    valid_since_year INTEGER,
    valid_until_semester semester,
    valid_until_year INTEGER,
    languages TEXT[] NOT NULL DEFAULT '{}',
    title TEXT NOT NULL,
    credits INTEGER NOT NULL DEFAULT 0,
    m_pord_nr INTEGER,
    m_p_nr INTEGER,
    mp_pord_nr INTEGER,
    mp_p_nr INTEGER,
    faculty_id INTEGER NOT NULL,
    institute_id INTEGER NOT NULL,
    fg_id INTEGER NOT NULL,
    responsible_id INTEGER NOT NULL,
    examination_board_id INTEGER NOT NULL,
    learning_result TEXT,
    content TEXT,
    teaching_information TEXT,
    max_attendees INTEGER,
    registration TEXT,
    duration INTEGER,
    requirements TEXT,
    additional_info TEXT,
    moses_link TEXT NOT NULL,
    PRIMARY KEY (id, version),
    CONSTRAINT fk_module_faculty FOREIGN KEY (faculty_id)
        REFERENCES faculty(id) ON DELETE RESTRICT,
    CONSTRAINT fk_module_institute FOREIGN KEY (institute_id)
        REFERENCES institute(id) ON DELETE RESTRICT,
    CONSTRAINT fk_module_fg FOREIGN KEY (fg_id)
        REFERENCES fachgebiet(id) ON DELETE RESTRICT,
    CONSTRAINT fk_module_responsible FOREIGN KEY (responsible_id)
        REFERENCES responsible_person(id) ON DELETE RESTRICT,
    CONSTRAINT fk_module_examination_board FOREIGN KEY (examination_board_id)
        REFERENCES examination_board(id) ON DELETE RESTRICT
);

-- ============================================================================
-- Create Dependent Tables
-- ============================================================================

CREATE TABLE contact (
    id SERIAL PRIMARY KEY,
    module_id INTEGER NOT NULL,
    module_version INTEGER NOT NULL,
    secretariat TEXT,
    contact_person TEXT,
    email TEXT,
    website TEXT,
    CONSTRAINT fk_contact_module FOREIGN KEY (module_id, module_version)
        REFERENCES module(id, version) ON DELETE CASCADE
);

CREATE TABLE exam (
    id SERIAL PRIMARY KEY,
    graded BOOLEAN NOT NULL,
    exam_type exam_type NOT NULL,
    clef TEXT,
    description TEXT
);

CREATE TABLE exam_component (
    id SERIAL PRIMARY KEY,
    exam_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    points INTEGER NOT NULL,
    category exam_category NOT NULL,
    scope TEXT,
    CONSTRAINT fk_exam_component_exam FOREIGN KEY (exam_id)
        REFERENCES exam(id) ON DELETE CASCADE
);

CREATE TABLE module_component (
    id SERIAL PRIMARY KEY,
    module_id INTEGER NOT NULL,
    module_version INTEGER NOT NULL,
    module_name TEXT,
    component_type component_type NOT NULL,
    number TEXT NOT NULL,
    rotation component_rotation NOT NULL,
    sws INTEGER NOT NULL,
    language TEXT NOT NULL,
    CONSTRAINT fk_module_component_module FOREIGN KEY (module_id, module_version)
        REFERENCES module(id, version) ON DELETE CASCADE
);

CREATE TABLE module_workload_distribution (
    id SERIAL PRIMARY KEY,
    module_id INTEGER NOT NULL,
    module_version INTEGER NOT NULL,
    description TEXT NOT NULL,
    factor DOUBLE PRECISION NOT NULL,
    hours DOUBLE PRECISION NOT NULL,
    total_hours DOUBLE PRECISION NOT NULL,
    CONSTRAINT fk_module_workload_module FOREIGN KEY (module_id, module_version)
        REFERENCES module(id, version) ON DELETE CASCADE
);

CREATE TABLE stupo (
    id SERIAL PRIMARY KEY,
    study_program_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    link TEXT NOT NULL,
    CONSTRAINT fk_stupo_study_program FOREIGN KEY (study_program_id)
        REFERENCES study_program(id) ON DELETE CASCADE,
    CONSTRAINT unique_stupo_per_program UNIQUE (study_program_id, name)
);

CREATE TABLE module_catalog_usage (
    id SERIAL PRIMARY KEY,
    stupo_id INTEGER NOT NULL,
    first_usage TEXT NOT NULL,
    last_usage TEXT NOT NULL,
    CONSTRAINT fk_module_catalog_usage_stupo FOREIGN KEY (stupo_id)
        REFERENCES stupo(id) ON DELETE CASCADE
);

-- ============================================================================
-- Create Indexes for Performance
-- ============================================================================

CREATE INDEX idx_module_faculty ON module(faculty_id);
CREATE INDEX idx_module_institute ON module(institute_id);
CREATE INDEX idx_module_fg ON module(fg_id);
CREATE INDEX idx_module_responsible ON module(responsible_id);
CREATE INDEX idx_module_examination_board ON module(examination_board_id);
CREATE INDEX idx_module_valid_since ON module(valid_since_year, valid_since_semester);

CREATE INDEX idx_contact_module ON contact(module_id, module_version);
CREATE INDEX idx_exam_component_exam ON exam_component(exam_id);
CREATE INDEX idx_module_component_module ON module_component(module_id, module_version);
CREATE INDEX idx_module_workload_module ON module_workload_distribution(module_id, module_version);
CREATE INDEX idx_stupo_study_program ON stupo(study_program_id);
CREATE INDEX idx_module_catalog_usage_stupo ON module_catalog_usage(stupo_id);
