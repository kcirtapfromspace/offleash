-- ============================================================================
-- Migration: Add Memberships for Multi-Context User Support
-- ============================================================================
-- This migration transforms the user model from single-org to multi-org capable.
-- A user (account) can now have multiple memberships across organizations with
-- different roles.
--
-- Workflows supported:
-- 1. User signs up → Can browse and book services (customer)
-- 2. User starts business → Creates org, becomes owner
-- 3. User joins business → Accepts invite, becomes walker/admin
-- 4. User can switch between contexts seamlessly
-- ============================================================================

-- Membership role enum (more granular than user_role)
CREATE TYPE membership_role AS ENUM ('owner', 'admin', 'walker', 'customer');

-- Membership status
CREATE TYPE membership_status AS ENUM ('active', 'suspended', 'pending', 'declined');

-- Memberships table - links users to organizations with roles
CREATE TABLE memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    role membership_role NOT NULL,
    status membership_status NOT NULL DEFAULT 'active',

    -- Metadata
    title VARCHAR(100),           -- Custom title like "Head Walker", "Owner"
    permissions JSONB DEFAULT '{}', -- Fine-grained permissions if needed

    -- Timestamps
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- A user can have multiple roles in the same org (e.g., owner AND walker)
    -- But not duplicate of same role
    UNIQUE(user_id, organization_id, role)
);

-- Indexes for efficient lookups
CREATE INDEX idx_memberships_user_id ON memberships(user_id);
CREATE INDEX idx_memberships_organization_id ON memberships(organization_id);
CREATE INDEX idx_memberships_user_org ON memberships(user_id, organization_id);
CREATE INDEX idx_memberships_role ON memberships(role);
CREATE INDEX idx_memberships_status ON memberships(status) WHERE status = 'active';

-- Active memberships view for common queries
CREATE INDEX idx_memberships_active ON memberships(user_id, status)
    WHERE status = 'active';

-- ============================================================================
-- Migrate existing user-organization relationships to memberships
-- ============================================================================

-- Create memberships from existing users table
INSERT INTO memberships (user_id, organization_id, role, status, joined_at, created_at)
SELECT
    id,
    organization_id,
    CASE
        WHEN role::text = 'admin' THEN 'owner'::membership_role
        WHEN role::text = 'walker' THEN 'walker'::membership_role
        WHEN role::text = 'customer' THEN 'customer'::membership_role
        ELSE 'customer'::membership_role
    END,
    'active'::membership_status,
    created_at,
    created_at
FROM users
WHERE organization_id IS NOT NULL;

-- ============================================================================
-- Update users table - make organization_id nullable for new architecture
-- ============================================================================

-- Make organization_id nullable (users can exist without org context)
ALTER TABLE users ALTER COLUMN organization_id DROP NOT NULL;

-- Add default_membership_id for quick context selection
ALTER TABLE users ADD COLUMN default_membership_id UUID REFERENCES memberships(id);

-- ============================================================================
-- Update invitations to reference memberships
-- ============================================================================

-- Add target_role to invitations (what role will the invitee have)
ALTER TABLE invitations ADD COLUMN target_role membership_role;

-- Update existing invitations
UPDATE invitations
SET target_role = CASE
    WHEN invitation_type = 'walker' THEN 'walker'::membership_role
    WHEN invitation_type = 'client' THEN 'customer'::membership_role
END;

-- Make target_role required for new invitations
ALTER TABLE invitations ALTER COLUMN target_role SET NOT NULL;

-- ============================================================================
-- Helper function to get user's active memberships
-- ============================================================================

CREATE OR REPLACE FUNCTION get_user_memberships(p_user_id UUID)
RETURNS TABLE (
    membership_id UUID,
    organization_id UUID,
    organization_name VARCHAR,
    organization_slug VARCHAR,
    role membership_role,
    status membership_status,
    title VARCHAR,
    joined_at TIMESTAMP WITH TIME ZONE
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        m.id,
        m.organization_id,
        o.name,
        o.slug,
        m.role,
        m.status,
        m.title,
        m.joined_at
    FROM memberships m
    JOIN organizations o ON o.id = m.organization_id
    WHERE m.user_id = p_user_id
      AND m.status = 'active'
    ORDER BY m.joined_at DESC;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Comments for documentation
-- ============================================================================

COMMENT ON TABLE memberships IS 'Links users to organizations with specific roles. Enables multi-org, multi-role user workflows.';
COMMENT ON COLUMN memberships.role IS 'owner: created the org, admin: full management, walker: service provider, customer: books services';
COMMENT ON COLUMN memberships.permissions IS 'Optional JSON for fine-grained permission overrides';
COMMENT ON COLUMN users.default_membership_id IS 'User''s preferred context for quick login';
