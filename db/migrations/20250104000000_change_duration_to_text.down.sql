-- Rollback duration from TEXT to INTEGER

ALTER TABLE module
ALTER COLUMN duration TYPE INTEGER USING (
    CASE
        WHEN duration ~ '^\d+$' THEN duration::INTEGER
        ELSE NULL
    END
);
