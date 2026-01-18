-- Add organization_id column to users table for multi-tenant scoping
-- Note: Column is nullable initially to allow for backfill of existing data
-- After backfill, a subsequent migration should add NOT NULL constraint

-- Add organization_id column
ALTER TABLE users ADD COLUMN organization_id UUID;

-- Add foreign key constraint to organizations
ALTER TABLE users ADD CONSTRAINT fk_users_organization
    FOREIGN KEY (organization_id) REFERENCES organizations(id);

-- Add index on organization_id for filtered queries
CREATE INDEX idx_users_organization_id ON users(organization_id);
