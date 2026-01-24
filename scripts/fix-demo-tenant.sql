-- =====================================================
-- OFFLEASH Demo Tenant Fix Script
-- =====================================================
-- Run this directly on your Render database:
-- 1. Go to Render dashboard > Database > Connect
-- 2. Paste and run these commands
-- =====================================================

-- First, let's check what exists
SELECT '=== Current Organization ===' as section;
SELECT id, name, slug, subdomain, settings
FROM organizations
WHERE slug = 'offleash-demo' OR id = '7af5b534-738f-45e1-829b-64689a85393f';

SELECT '=== Current Tenant Database ===' as section;
SELECT td.*, o.name as org_name
FROM tenant_databases td
LEFT JOIN organizations o ON o.id = td.organization_id
WHERE td.organization_id = '7af5b534-738f-45e1-829b-64689a85393f';

SELECT '=== Current Organization Settings ===' as section;
SELECT os.*, o.name as org_name
FROM organization_settings os
LEFT JOIN organizations o ON o.id = os.organization_id
WHERE os.organization_id = '7af5b534-738f-45e1-829b-64689a85393f';

SELECT '=== Demo User Memberships ===' as section;
SELECT u.email, m.role, m.status, o.name as org_name
FROM memberships m
JOIN users u ON u.id = m.user_id
JOIN organizations o ON o.id = m.organization_id
WHERE u.email IN ('customer@demo.com', 'alex@demo.com', 'admin@demo.com', 'e2e@test.com');

-- =====================================================
-- FIX: Add missing tenant_databases entry
-- =====================================================
INSERT INTO tenant_databases (organization_id, connection_string, status)
VALUES (
    '7af5b534-738f-45e1-829b-64689a85393f',
    '',
    'active'
)
ON CONFLICT (organization_id) DO UPDATE SET
    status = 'active',
    updated_at = NOW();

-- =====================================================
-- FIX: Add missing organization_settings entry
-- =====================================================
INSERT INTO organization_settings (organization_id, branding, feature_flags, subscription_tier)
VALUES (
    '7af5b534-738f-45e1-829b-64689a85393f',
    '{}',
    '{"allow_customer_booking": true}',
    'starter'
)
ON CONFLICT (organization_id) DO UPDATE SET
    subscription_tier = 'starter',
    updated_at = NOW();

-- =====================================================
-- FIX: Update organization settings with branding
-- =====================================================
UPDATE organizations
SET settings = '{"primary_color": "#4f46e5", "secondary_color": "#10b981", "font_family": "Inter"}'::jsonb
WHERE id = '7af5b534-738f-45e1-829b-64689a85393f'
  AND (settings IS NULL OR settings = '{}');

-- =====================================================
-- VERIFY: Check everything is set up
-- =====================================================
SELECT '=== AFTER FIX: Tenant Database ===' as section;
SELECT td.id, td.status, o.name
FROM tenant_databases td
JOIN organizations o ON o.id = td.organization_id
WHERE td.organization_id = '7af5b534-738f-45e1-829b-64689a85393f';

SELECT '=== AFTER FIX: Organization Settings ===' as section;
SELECT os.subscription_tier, o.name, o.settings
FROM organization_settings os
JOIN organizations o ON o.id = os.organization_id
WHERE os.organization_id = '7af5b534-738f-45e1-829b-64689a85393f';

SELECT '=== Services Available ===' as section;
SELECT name, is_active, base_price_cents/100.0 as price_dollars
FROM services
WHERE organization_id = '7af5b534-738f-45e1-829b-64689a85393f';
