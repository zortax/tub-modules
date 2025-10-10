-- Change duration from INTEGER to TEXT to store full section content

ALTER TABLE module
ALTER COLUMN duration TYPE TEXT USING duration::TEXT;
