-- Create tenant database status enum
CREATE TYPE tenant_db_status AS ENUM ('active', 'inactive', 'provisioning');

-- Create tenant_databases table for tracking tenant database connections
CREATE TABLE tenant_databases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL UNIQUE REFERENCES organizations(id) ON DELETE CASCADE,
    connection_string VARCHAR(500) NOT NULL, -- Should be encrypted at application level
    status tenant_db_status NOT NULL DEFAULT 'provisioning',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for status-based queries
CREATE INDEX idx_tenant_databases_status ON tenant_databases(status);
