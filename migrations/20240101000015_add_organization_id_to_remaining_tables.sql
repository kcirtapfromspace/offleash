-- Add organization_id column to remaining business tables for multi-tenant scoping
-- Tables: locations, bookings, payments, blocks, working_hours
-- Note: Columns are nullable initially to allow for backfill of existing data
-- After backfill, a subsequent migration should add NOT NULL constraints

-- ============================================================================
-- LOCATIONS TABLE
-- ============================================================================

-- Add organization_id column
ALTER TABLE locations ADD COLUMN organization_id UUID;

-- Add foreign key constraint to organizations
ALTER TABLE locations ADD CONSTRAINT fk_locations_organization
    FOREIGN KEY (organization_id) REFERENCES organizations(id);

-- Add index on organization_id for filtered queries
CREATE INDEX idx_locations_organization_id ON locations(organization_id);

-- ============================================================================
-- BOOKINGS TABLE
-- ============================================================================

-- Add organization_id column
ALTER TABLE bookings ADD COLUMN organization_id UUID;

-- Add foreign key constraint to organizations
ALTER TABLE bookings ADD CONSTRAINT fk_bookings_organization
    FOREIGN KEY (organization_id) REFERENCES organizations(id);

-- Add index on organization_id for filtered queries
CREATE INDEX idx_bookings_organization_id ON bookings(organization_id);

-- ============================================================================
-- PAYMENTS TABLE
-- ============================================================================

-- Add organization_id column
ALTER TABLE payments ADD COLUMN organization_id UUID;

-- Add foreign key constraint to organizations
ALTER TABLE payments ADD CONSTRAINT fk_payments_organization
    FOREIGN KEY (organization_id) REFERENCES organizations(id);

-- Add index on organization_id for filtered queries
CREATE INDEX idx_payments_organization_id ON payments(organization_id);

-- ============================================================================
-- BLOCKS TABLE
-- ============================================================================

-- Add organization_id column
ALTER TABLE blocks ADD COLUMN organization_id UUID;

-- Add foreign key constraint to organizations
ALTER TABLE blocks ADD CONSTRAINT fk_blocks_organization
    FOREIGN KEY (organization_id) REFERENCES organizations(id);

-- Add index on organization_id for filtered queries
CREATE INDEX idx_blocks_organization_id ON blocks(organization_id);

-- ============================================================================
-- WORKING_HOURS TABLE
-- ============================================================================

-- Add organization_id column
ALTER TABLE working_hours ADD COLUMN organization_id UUID;

-- Add foreign key constraint to organizations
ALTER TABLE working_hours ADD CONSTRAINT fk_working_hours_organization
    FOREIGN KEY (organization_id) REFERENCES organizations(id);

-- Add index on organization_id for filtered queries
CREATE INDEX idx_working_hours_organization_id ON working_hours(organization_id);
