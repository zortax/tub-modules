-- Revert component_type from TEXT back to ENUM
-- WARNING: This will lose any component types that are not VL, UE, or PJ

-- Step 1: Recreate the enum type
CREATE TYPE component_type AS ENUM ('VL', 'UE', 'PJ');

-- Step 2: Add a new enum column
ALTER TABLE module_component ADD COLUMN component_type_enum component_type;

-- Step 3: Copy data from text to enum, defaulting to 'VL' for unknown types
UPDATE module_component
SET component_type_enum = CASE
    WHEN component_type IN ('VL', 'UE', 'PJ') THEN component_type::component_type
    ELSE 'VL'::component_type
END;

-- Step 4: Make the new column NOT NULL
ALTER TABLE module_component ALTER COLUMN component_type_enum SET NOT NULL;

-- Step 5: Drop the old text column
ALTER TABLE module_component DROP COLUMN component_type;

-- Step 6: Rename the new column to component_type
ALTER TABLE module_component RENAME COLUMN component_type_enum TO component_type;
