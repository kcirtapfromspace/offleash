-- Add first_name and last_name columns to platform_admins table
-- These were missing from the original migration

ALTER TABLE platform_admins
ADD COLUMN IF NOT EXISTS first_name VARCHAR(100) NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS last_name VARCHAR(100) NOT NULL DEFAULT '';
