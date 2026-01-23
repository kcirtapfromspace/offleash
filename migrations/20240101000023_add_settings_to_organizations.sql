-- Add settings column to organizations table for branding configuration
ALTER TABLE organizations
ADD COLUMN settings JSONB NOT NULL DEFAULT '{}';
