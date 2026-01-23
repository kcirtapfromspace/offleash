-- Add settings column to organizations table for branding configuration
ALTER TABLE organizations
ADD COLUMN IF NOT EXISTS settings JSONB NOT NULL DEFAULT '{}';
