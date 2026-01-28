-- =====================================================
-- OFFLEASH E2E Test Seed Data
-- =====================================================
-- Creates test data matching tests/utils/constants.ts
-- Run: psql -d offleash_test -f tests/fixtures/seed.sql
-- =====================================================

BEGIN;

-- =====================================================
-- Clean up existing test data (if running multiple times)
-- =====================================================
DELETE FROM bookings WHERE organization_id IN (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000002'
);
DELETE FROM locations WHERE organization_id IN (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000002'
);
DELETE FROM working_hours WHERE organization_id IN (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000002'
);
DELETE FROM services WHERE organization_id IN (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000002'
);
DELETE FROM memberships WHERE organization_id IN (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000002'
);
DELETE FROM users WHERE email LIKE '%@test.offleash.world' OR email LIKE '%@org-b.test';
DELETE FROM organizations WHERE id IN (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000002'
);
DELETE FROM platform_admins WHERE email = 'platform@test.offleash.world';

-- =====================================================
-- Organizations
-- =====================================================
INSERT INTO organizations (id, name, slug, subdomain, settings, created_at, updated_at)
VALUES
    (
        '00000000-0000-0000-0000-000000000001',
        'OFFLEASH Demo',
        'offleash-demo',
        'demo',
        '{"branding": {"primary_color": "#10B981", "business_name": "OFFLEASH Demo"}}'::jsonb,
        NOW(),
        NOW()
    ),
    (
        '00000000-0000-0000-0000-000000000002',
        'Test Org B',
        'test-org-b',
        'orgb',
        '{"branding": {"primary_color": "#3B82F6", "business_name": "Test Org B"}}'::jsonb,
        NOW(),
        NOW()
    );

-- =====================================================
-- Tenant Databases (point to same test database)
-- =====================================================
INSERT INTO tenant_databases (id, organization_id, connection_string, status, created_at, updated_at)
VALUES
    (
        '70000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'postgres://localhost:5432/offleash_test',
        'active',
        NOW(),
        NOW()
    ),
    (
        '70000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000002',
        'postgres://localhost:5432/offleash_test',
        'active',
        NOW(),
        NOW()
    );

-- =====================================================
-- Test Users
-- Password for all: TestPassword123!
-- Hash: $argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpamtsbW5vcA$rKzFQfWVTxP8ywKYjVqQm8JgBB8qW3OhYKMLvQCzYxQ
-- =====================================================

-- Customer for Demo Org
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    '10000000-0000-0000-0000-000000000001',
    'customer@test.offleash.world',
    '$argon2id$v=19$m=19456,t=2,p=1$2agxGhWPi/pudQ3fbIqnfQ$m0apOXuqbHxOH2TSb/y6gB5gACzAI3YH7J95KgeibPc',
    'customer',
    'Test',
    'Customer',
    '555-100-0001',
    '00000000-0000-0000-0000-000000000001',
    NOW(),
    NOW()
);

-- Walker for Demo Org
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    '20000000-0000-0000-0000-000000000001',
    'walker@test.offleash.world',
    '$argon2id$v=19$m=19456,t=2,p=1$2agxGhWPi/pudQ3fbIqnfQ$m0apOXuqbHxOH2TSb/y6gB5gACzAI3YH7J95KgeibPc',
    'walker',
    'Test',
    'Walker',
    '555-200-0001',
    '00000000-0000-0000-0000-000000000001',
    NOW(),
    NOW()
);

-- Admin for Demo Org
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    '30000000-0000-0000-0000-000000000001',
    'admin@test.offleash.world',
    '$argon2id$v=19$m=19456,t=2,p=1$2agxGhWPi/pudQ3fbIqnfQ$m0apOXuqbHxOH2TSb/y6gB5gACzAI3YH7J95KgeibPc',
    'admin',
    'Test',
    'Admin',
    '555-300-0001',
    '00000000-0000-0000-0000-000000000001',
    NOW(),
    NOW()
);

-- Owner for Demo Org
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    '40000000-0000-0000-0000-000000000001',
    'owner@test.offleash.world',
    '$argon2id$v=19$m=19456,t=2,p=1$2agxGhWPi/pudQ3fbIqnfQ$m0apOXuqbHxOH2TSb/y6gB5gACzAI3YH7J95KgeibPc',
    'admin',
    'Test',
    'Owner',
    '555-400-0001',
    '00000000-0000-0000-0000-000000000001',
    NOW(),
    NOW()
);

-- Customer for Org B (isolation testing)
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    '50000000-0000-0000-0000-000000000001',
    'customer@org-b.test',
    '$argon2id$v=19$m=19456,t=2,p=1$2agxGhWPi/pudQ3fbIqnfQ$m0apOXuqbHxOH2TSb/y6gB5gACzAI3YH7J95KgeibPc',
    'customer',
    'OrgB',
    'Customer',
    '555-500-0001',
    '00000000-0000-0000-0000-000000000002',
    NOW(),
    NOW()
);

-- =====================================================
-- Memberships (multi-org context support)
-- =====================================================
INSERT INTO memberships (id, user_id, organization_id, role, status, joined_at, created_at, updated_at)
VALUES
    -- Customer in Demo Org
    (
        '11000000-0000-0000-0000-000000000001',
        '10000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'customer',
        'active',
        NOW(),
        NOW(),
        NOW()
    ),
    -- Walker in Demo Org
    (
        '22000000-0000-0000-0000-000000000001',
        '20000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'walker',
        'active',
        NOW(),
        NOW(),
        NOW()
    ),
    -- Admin in Demo Org
    (
        '33000000-0000-0000-0000-000000000001',
        '30000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'admin',
        'active',
        NOW(),
        NOW(),
        NOW()
    ),
    -- Owner in Demo Org
    (
        '44000000-0000-0000-0000-000000000001',
        '40000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'owner',
        'active',
        NOW(),
        NOW(),
        NOW()
    ),
    -- Customer in Org B
    (
        '55000000-0000-0000-0000-000000000001',
        '50000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000002',
        'customer',
        'active',
        NOW(),
        NOW(),
        NOW()
    );

-- Set default memberships
UPDATE users SET default_membership_id = '11000000-0000-0000-0000-000000000001' WHERE id = '10000000-0000-0000-0000-000000000001';
UPDATE users SET default_membership_id = '22000000-0000-0000-0000-000000000001' WHERE id = '20000000-0000-0000-0000-000000000001';
UPDATE users SET default_membership_id = '33000000-0000-0000-0000-000000000001' WHERE id = '30000000-0000-0000-0000-000000000001';
UPDATE users SET default_membership_id = '44000000-0000-0000-0000-000000000001' WHERE id = '40000000-0000-0000-0000-000000000001';
UPDATE users SET default_membership_id = '55000000-0000-0000-0000-000000000001' WHERE id = '50000000-0000-0000-0000-000000000001';

-- =====================================================
-- Platform Admin
-- =====================================================
INSERT INTO platform_admins (id, email, password_hash, first_name, last_name, created_at, updated_at)
VALUES (
    '90000000-0000-0000-0000-000000000001',
    'platform@test.offleash.world',
    '$argon2id$v=19$m=19456,t=2,p=1$2agxGhWPi/pudQ3fbIqnfQ$m0apOXuqbHxOH2TSb/y6gB5gACzAI3YH7J95KgeibPc',
    'Platform',
    'Admin',
    NOW(),
    NOW()
);

-- =====================================================
-- Services for Demo Org
-- =====================================================
INSERT INTO services (id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at)
VALUES
    (
        '60000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        '30 Minute Walk',
        'A quick 30-minute walk for your pup',
        30,
        2500,
        true,
        NOW(),
        NOW()
    ),
    (
        '60000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000001',
        '60 Minute Walk',
        'A full hour walk with playtime',
        60,
        4000,
        true,
        NOW(),
        NOW()
    ),
    (
        '60000000-0000-0000-0000-000000000003',
        '00000000-0000-0000-0000-000000000001',
        'Pet Sitting',
        'In-home pet sitting for extended periods',
        120,
        5000,
        true,
        NOW(),
        NOW()
    );

-- Services for Org B (to verify isolation)
INSERT INTO services (id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at)
VALUES
    (
        '60000000-0000-0000-0000-000000000101',
        '00000000-0000-0000-0000-000000000002',
        'Org B Walk',
        'Service specific to Org B',
        30,
        3000,
        true,
        NOW(),
        NOW()
    );

-- =====================================================
-- Customer Locations
-- =====================================================
INSERT INTO locations (id, user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at)
VALUES
    (
        '70000000-0000-0000-0000-000000000001',
        '10000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'Home',
        '123 Test Street',
        'Denver',
        'CO',
        '80202',
        39.7392,
        -104.9903,
        'Ring doorbell, dog is friendly',
        true,
        NOW(),
        NOW()
    ),
    (
        '70000000-0000-0000-0000-000000000002',
        '10000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'Work',
        '456 Work Avenue',
        'Denver',
        'CO',
        '80203',
        39.7312,
        -104.9826,
        'Meet in lobby',
        false,
        NOW(),
        NOW()
    ),
    -- Org B customer location
    (
        '70000000-0000-0000-0000-000000000101',
        '50000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000002',
        'Home',
        '789 OrgB Street',
        'Boulder',
        'CO',
        '80301',
        40.0150,
        -105.2705,
        'Org B customer location',
        true,
        NOW(),
        NOW()
    );

-- =====================================================
-- Walker Working Hours (Mon-Sun 8am-6pm)
-- =====================================================
INSERT INTO working_hours (id, walker_id, organization_id, day_of_week, start_time, end_time, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(),
    '20000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000001',
    dow,
    '08:00:00'::time,
    '18:00:00'::time,
    true,
    NOW(),
    NOW()
FROM generate_series(0, 6) AS dow;

-- =====================================================
-- Sample Bookings
-- =====================================================
INSERT INTO bookings (id, customer_id, walker_id, service_id, location_id, organization_id, status, scheduled_start, scheduled_end, price_cents, notes, created_at, updated_at)
VALUES
    -- Confirmed booking for tomorrow
    (
        '80000000-0000-0000-0000-000000000001',
        '10000000-0000-0000-0000-000000000001',
        '20000000-0000-0000-0000-000000000001',
        '60000000-0000-0000-0000-000000000001',
        '70000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'confirmed',
        (CURRENT_DATE + INTERVAL '1 day')::date + TIME '10:00:00',
        (CURRENT_DATE + INTERVAL '1 day')::date + TIME '10:30:00',
        2500,
        'Test booking for tomorrow',
        NOW(),
        NOW()
    ),
    -- Pending booking for day after tomorrow
    (
        '80000000-0000-0000-0000-000000000002',
        '10000000-0000-0000-0000-000000000001',
        '20000000-0000-0000-0000-000000000001',
        '60000000-0000-0000-0000-000000000002',
        '70000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'pending',
        (CURRENT_DATE + INTERVAL '2 days')::date + TIME '14:00:00',
        (CURRENT_DATE + INTERVAL '2 days')::date + TIME '15:00:00',
        4000,
        'Test booking pending approval',
        NOW(),
        NOW()
    ),
    -- Completed booking from yesterday
    (
        '80000000-0000-0000-0000-000000000003',
        '10000000-0000-0000-0000-000000000001',
        '20000000-0000-0000-0000-000000000001',
        '60000000-0000-0000-0000-000000000001',
        '70000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        'completed',
        (CURRENT_DATE - INTERVAL '1 day')::date + TIME '10:00:00',
        (CURRENT_DATE - INTERVAL '1 day')::date + TIME '10:30:00',
        2500,
        'Completed test booking',
        NOW() - INTERVAL '1 day',
        NOW()
    );

COMMIT;

-- =====================================================
-- Verification
-- =====================================================
\echo ''
\echo '============================================='
\echo 'Test Data Seeding Complete!'
\echo '============================================='
\echo ''

\echo '=== Organizations ==='
SELECT id, name, slug FROM organizations WHERE id IN ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000002');

\echo ''
\echo '=== Test Users ==='
SELECT email, role, first_name || ' ' || last_name as name
FROM users
WHERE email LIKE '%@test.offleash.world' OR email LIKE '%@org-b.test'
ORDER BY email;

\echo ''
\echo '=== Memberships ==='
SELECT u.email, o.slug as org, m.role, m.status
FROM memberships m
JOIN users u ON m.user_id = u.id
JOIN organizations o ON m.organization_id = o.id
WHERE m.status = 'active'
ORDER BY o.slug, m.role;

\echo ''
\echo '=== Services (Demo Org) ==='
SELECT name, duration_minutes || ' min' as duration, '$' || (base_price_cents / 100.0)::text as price
FROM services
WHERE organization_id = '00000000-0000-0000-0000-000000000001' AND is_active = true;

\echo ''
\echo '=== Platform Admin ==='
SELECT email FROM platform_admins WHERE email = 'platform@test.offleash.world';

\echo ''
\echo '============================================='
\echo 'Test Credentials:'
\echo '  customer@test.offleash.world / TestPassword123!'
\echo '  walker@test.offleash.world / TestPassword123!'
\echo '  admin@test.offleash.world / TestPassword123!'
\echo '  owner@test.offleash.world / TestPassword123!'
\echo '  platform@test.offleash.world / TestPassword123!'
\echo '  customer@org-b.test / TestPassword123! (Org B)'
\echo '============================================='
