-- Remove 'praktisch' from exam_category enum
-- Note: This will fail if any rows use 'praktisch'
-- You may need to update those rows first

-- PostgreSQL doesn't support removing enum values directly
-- The safest way is to recreate the enum, but that requires dropping and recreating
-- all tables/columns that use it. For now, we'll just document this limitation.

-- To properly rollback, you would need to:
-- 1. Update all exam_component rows with category='praktisch' to another value
-- 2. Create a new enum without 'praktisch'
-- 3. Alter the table to use the new enum
-- 4. Drop the old enum

-- For simplicity, this migration cannot be automatically rolled back
