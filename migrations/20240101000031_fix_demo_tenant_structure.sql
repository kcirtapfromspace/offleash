-- =====================================================
-- Fix Demo Tenant Structure
-- =====================================================
-- This migration ensures the demo organization has all
-- the required tenant infrastructure:
-- 1. tenant_databases entry (with status 'active')
-- 2. organization_settings entry (with subscription tier)
-- 3. Proper organization settings JSONB
-- =====================================================

-- Demo organization ID (from seed-test-data.sql)
-- '7af5b534-738f-45e1-829b-64689a85393f'

-- =====================================================
-- Step 1: Ensure tenant_databases entry exists
-- =====================================================
INSERT INTO tenant_databases (organization_id, connection_string, status)
SELECT
    '7af5b534-738f-45e1-829b-64689a85393f'::uuid,
    '', -- Empty connection string (single-DB mode)
    'active'::tenant_db_status
WHERE EXISTS (
    SELECT 1 FROM organizations WHERE id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
)
ON CONFLICT (organization_id) DO UPDATE SET
    status = 'active'::tenant_db_status,
    updated_at = NOW();

-- =====================================================
-- Step 2: Ensure organization_settings entry exists
-- =====================================================
INSERT INTO organization_settings (organization_id, branding, feature_flags, subscription_tier)
SELECT
    '7af5b534-738f-45e1-829b-64689a85393f'::uuid,
    '{}'::jsonb,
    '{"allow_customer_booking": true, "allow_walker_scheduling": true}'::jsonb,
    'starter'
WHERE EXISTS (
    SELECT 1 FROM organizations WHERE id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
)
ON CONFLICT (organization_id) DO UPDATE SET
    subscription_tier = 'starter',
    updated_at = NOW();

-- =====================================================
-- Step 3: Update organization settings JSONB with branding
-- =====================================================
UPDATE organizations
SET settings = jsonb_build_object(
    'primary_color', '#4f46e5',
    'secondary_color', '#10b981',
    'font_family', 'Inter'
)
WHERE id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
  AND (settings IS NULL OR settings = '{}'::jsonb);

-- =====================================================
-- Step 4: Verify memberships exist for demo users
-- =====================================================
-- The memberships migration should have created these, but ensure they exist

-- Customer membership
INSERT INTO memberships (user_id, organization_id, role, status)
SELECT
    'de000000-0000-0000-0000-000000000001'::uuid,
    '7af5b534-738f-45e1-829b-64689a85393f'::uuid,
    'customer'::membership_role,
    'active'::membership_status
WHERE EXISTS (SELECT 1 FROM users WHERE id = 'de000000-0000-0000-0000-000000000001'::uuid)
ON CONFLICT (user_id, organization_id, role) DO NOTHING;

-- Walker membership
INSERT INTO memberships (user_id, organization_id, role, status)
SELECT
    'b376c762-b772-4fde-963e-5dcaedd52626'::uuid,
    '7af5b534-738f-45e1-829b-64689a85393f'::uuid,
    'walker'::membership_role,
    'active'::membership_status
WHERE EXISTS (SELECT 1 FROM users WHERE id = 'b376c762-b772-4fde-963e-5dcaedd52626'::uuid)
ON CONFLICT (user_id, organization_id, role) DO NOTHING;

-- Admin/Owner membership
INSERT INTO memberships (user_id, organization_id, role, status)
SELECT
    'ad000000-0000-0000-0000-000000000001'::uuid,
    '7af5b534-738f-45e1-829b-64689a85393f'::uuid,
    'owner'::membership_role,
    'active'::membership_status
WHERE EXISTS (SELECT 1 FROM users WHERE id = 'ad000000-0000-0000-0000-000000000001'::uuid)
ON CONFLICT (user_id, organization_id, role) DO NOTHING;

-- E2E test customer membership
INSERT INTO memberships (user_id, organization_id, role, status)
SELECT
    'e2e00000-0000-0000-0000-000000000001'::uuid,
    '7af5b534-738f-45e1-829b-64689a85393f'::uuid,
    'customer'::membership_role,
    'active'::membership_status
WHERE EXISTS (SELECT 1 FROM users WHERE id = 'e2e00000-0000-0000-0000-000000000001'::uuid)
ON CONFLICT (user_id, organization_id, role) DO NOTHING;
