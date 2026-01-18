-- Add organization_id column to services table for multi-tenant scoping
-- Note: Column is nullable initially to allow for backfill of existing data
-- After backfill, a subsequent migration should add NOT NULL constraint

-- Add organization_id column
ALTER TABLE services ADD COLUMN organization_id UUID;

-- Add foreign key constraint to organizations
ALTER TABLE services ADD CONSTRAINT fk_services_organization
    FOREIGN KEY (organization_id) REFERENCES organizations(id);

-- Add index on organization_id for filtered queries
CREATE INDEX idx_services_organization_id ON services(organization_id);
