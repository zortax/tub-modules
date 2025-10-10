-- Change component_type from ENUM to TEXT to support arbitrary component types
-- This allows for component types beyond VL, UE, and PJ (e.g., SEM, PR, etc.)

-- Step 1: Add a new TEXT column
ALTER TABLE module_component ADD COLUMN component_type_text TEXT;

-- Step 2: Copy data from enum to text
UPDATE module_component SET component_type_text = component_type::text;

-- Step 3: Make the new column NOT NULL
ALTER TABLE module_component ALTER COLUMN component_type_text SET NOT NULL;

-- Step 4: Drop the old enum column
ALTER TABLE module_component DROP COLUMN component_type;

-- Step 5: Rename the new column to component_type
ALTER TABLE module_component RENAME COLUMN component_type_text TO component_type;

-- Step 6: Drop the old enum type
DROP TYPE component_type;
