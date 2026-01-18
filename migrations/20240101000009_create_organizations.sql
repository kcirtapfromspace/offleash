-- Create organizations table for multi-tenant support
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    subdomain VARCHAR(100) NOT NULL,
    custom_domain VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on slug for fast tenant lookups
CREATE INDEX idx_organizations_slug ON organizations(slug);

-- Index on subdomain for domain-based routing
CREATE INDEX idx_organizations_subdomain ON organizations(subdomain);
