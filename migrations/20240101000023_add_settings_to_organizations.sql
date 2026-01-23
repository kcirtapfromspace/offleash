-- Add settings column to organizations table for branding configuration
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'organizations' AND column_name = 'settings'
    ) THEN
        ALTER TABLE organizations ADD COLUMN settings JSONB NOT NULL DEFAULT '{}';
    END IF;
END $$;
