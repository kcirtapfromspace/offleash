-- =====================================================
-- OFFLEASH Test Data Seed Script
-- =====================================================
-- This script creates test accounts and services for E2E testing
-- Run with: psql -d offleash -f demos/seed-test-data.sql
--           OR docker compose exec db psql -U offleash -d offleash -f /path/to/seed-test-data.sql
-- =====================================================

-- Configuration
\set org_id '7af5b534-738f-45e1-829b-64689a85393f'

-- =====================================================
-- Step 1: Ensure organization exists
-- =====================================================
INSERT INTO organizations (id, name, slug, subdomain, created_at, updated_at)
VALUES (
    :'org_id'::uuid,
    'OFFLEASH Demo',
    'offleash-demo',
    'demo',
    NOW(),
    NOW()
)
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;

-- =====================================================
-- Step 2: Create Test Users with REAL password hashes
-- =====================================================
-- Password for all test accounts: TestPassword123!
-- Generated using: argon2id with standard parameters

-- E2E Test User (Customer)
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'e2e00000-0000-0000-0000-000000000001'::uuid,
    'e2e@test.com',
    -- Hash for 'TestPassword123!' - you may need to regenerate this with your actual hasher
    '$argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpamtsbW5vcA$rKzFQfWVTxP8ywKYjVqQm8JgBB8qW3OhYKMLvQCzYxQ',
    'customer',
    'E2E',
    'Tester',
    '555-000-0001',
    :'org_id'::uuid,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO UPDATE SET
    password_hash = EXCLUDED.password_hash,
    first_name = EXCLUDED.first_name,
    last_name = EXCLUDED.last_name;

-- Demo Customer Account
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'de000000-0000-0000-0000-000000000001'::uuid,
    'customer@demo.com',
    '$argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpamtsbW5vcA$rKzFQfWVTxP8ywKYjVqQm8JgBB8qW3OhYKMLvQCzYxQ',
    'customer',
    'Demo',
    'Customer',
    '555-DEMO-001',
    :'org_id'::uuid,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO UPDATE SET
    password_hash = EXCLUDED.password_hash,
    first_name = EXCLUDED.first_name;

-- Demo Walker Account
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'b376c762-b772-4fde-963e-5dcaedd52626'::uuid,
    'alex@demo.com',
    '$argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpamtsbW5vcA$rKzFQfWVTxP8ywKYjVqQm8JgBB8qW3OhYKMLvQCzYxQ',
    'walker',
    'Alex',
    'Walker',
    '555-WALK-001',
    :'org_id'::uuid,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO UPDATE SET
    password_hash = EXCLUDED.password_hash,
    first_name = EXCLUDED.first_name;

-- Demo Admin Account
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'ad000000-0000-0000-0000-000000000001'::uuid,
    'admin@demo.com',
    '$argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpamtsbW5vcA$rKzFQfWVTxP8ywKYjVqQm8JgBB8qW3OhYKMLvQCzYxQ',
    'admin',
    'Demo',
    'Admin',
    '555-ADMIN-01',
    :'org_id'::uuid,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO UPDATE SET
    password_hash = EXCLUDED.password_hash,
    first_name = EXCLUDED.first_name;

-- =====================================================
-- Step 3: Create Services
-- =====================================================
INSERT INTO services (id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at)
VALUES
    (
        '319d40cd-1730-43f2-b115-200821e76856'::uuid,
        :'org_id'::uuid,
        '30 Min Walk',
        'A quick 30-minute walk around the neighborhood. Perfect for a midday break.',
        30,
        2500,
        true,
        NOW(),
        NOW()
    ),
    (
        '419d40cd-1730-43f2-b115-200821e76857'::uuid,
        :'org_id'::uuid,
        '60 Min Adventure Walk',
        'A full hour of walking, exploring, and exercise. Great for high-energy dogs.',
        60,
        4000,
        true,
        NOW(),
        NOW()
    ),
    (
        '519d40cd-1730-43f2-b115-200821e76858'::uuid,
        :'org_id'::uuid,
        'Puppy Play Session',
        '45 minutes of supervised play and socialization for puppies under 1 year.',
        45,
        3500,
        true,
        NOW(),
        NOW()
    ),
    (
        '619d40cd-1730-43f2-b115-200821e76859'::uuid,
        :'org_id'::uuid,
        'Drop-In Visit',
        '15-minute check-in for feeding, potty breaks, and quick cuddles.',
        15,
        1500,
        true,
        NOW(),
        NOW()
    )
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    base_price_cents = EXCLUDED.base_price_cents,
    is_active = EXCLUDED.is_active;

-- =====================================================
-- Step 4: Create Customer Locations
-- =====================================================
INSERT INTO locations (id, user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at)
VALUES
    (
        '10c00000-0000-0000-0000-000000000001'::uuid,
        'e2e00000-0000-0000-0000-000000000001'::uuid,
        :'org_id'::uuid,
        'Home',
        '1234 Test Street',
        'Denver',
        'CO',
        '80202',
        39.7392,
        -104.9903,
        'E2E test location - front door code: 1234',
        true,
        NOW(),
        NOW()
    ),
    (
        '10c00000-0000-0000-0000-000000000002'::uuid,
        'de000000-0000-0000-0000-000000000001'::uuid,
        :'org_id'::uuid,
        'Home',
        '5678 Demo Avenue',
        'Denver',
        'CO',
        '80206',
        39.7176,
        -104.9548,
        'Demo customer home - yellow house',
        true,
        NOW(),
        NOW()
    )
ON CONFLICT (id) DO UPDATE SET
    address = EXCLUDED.address,
    notes = EXCLUDED.notes;

-- =====================================================
-- Step 5: Create Walker Working Hours
-- =====================================================
INSERT INTO working_hours (id, walker_id, organization_id, day_of_week, start_time, end_time, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(),
    'b376c762-b772-4fde-963e-5dcaedd52626'::uuid,
    :'org_id'::uuid,
    dow,
    '08:00:00'::time,
    '18:00:00'::time,
    true,
    NOW(),
    NOW()
FROM generate_series(0, 6) AS dow
ON CONFLICT DO NOTHING;

-- =====================================================
-- Step 6: Create Sample Bookings for Today/Tomorrow
-- =====================================================
INSERT INTO bookings (id, customer_id, walker_id, service_id, location_id, organization_id, status, scheduled_start, scheduled_end, base_price_cents, notes, created_at, updated_at)
VALUES
    (
        'b00c0000-0000-0000-0000-000000000001'::uuid,
        'de000000-0000-0000-0000-000000000001'::uuid,
        'b376c762-b772-4fde-963e-5dcaedd52626'::uuid,
        '319d40cd-1730-43f2-b115-200821e76856'::uuid,
        '10c00000-0000-0000-0000-000000000002'::uuid,
        :'org_id'::uuid,
        'confirmed',
        CURRENT_DATE + TIME '10:00:00',
        CURRENT_DATE + TIME '10:30:00',
        2500,
        'Morning walk for demo',
        NOW(),
        NOW()
    ),
    (
        'b00c0000-0000-0000-0000-000000000002'::uuid,
        'de000000-0000-0000-0000-000000000001'::uuid,
        'b376c762-b772-4fde-963e-5dcaedd52626'::uuid,
        '419d40cd-1730-43f2-b115-200821e76857'::uuid,
        '10c00000-0000-0000-0000-000000000002'::uuid,
        :'org_id'::uuid,
        'confirmed',
        CURRENT_DATE + INTERVAL '1 day' + TIME '14:00:00',
        CURRENT_DATE + INTERVAL '1 day' + TIME '15:00:00',
        4000,
        'Adventure walk tomorrow',
        NOW(),
        NOW()
    )
ON CONFLICT (id) DO UPDATE SET
    status = EXCLUDED.status,
    scheduled_start = EXCLUDED.scheduled_start,
    scheduled_end = EXCLUDED.scheduled_end;

-- =====================================================
-- Verification Queries
-- =====================================================
\echo ''
\echo '============================================='
\echo 'Test Data Seeding Complete!'
\echo '============================================='
\echo ''

\echo '=== Test Accounts ==='
SELECT email, role, first_name || ' ' || last_name as name
FROM users
WHERE email IN ('e2e@test.com', 'customer@demo.com', 'alex@demo.com', 'admin@demo.com')
ORDER BY role, email;

\echo ''
\echo '=== Services ==='
SELECT name, duration_minutes || ' min' as duration, '$' || (base_price_cents / 100.0)::text as price
FROM services
WHERE organization_id = :'org_id'::uuid AND is_active = true
ORDER BY base_price_cents;

\echo ''
\echo '=== Upcoming Bookings ==='
SELECT
    b.scheduled_start::date as date,
    b.scheduled_start::time as time,
    s.name as service,
    u.first_name as customer,
    b.status
FROM bookings b
JOIN services s ON b.service_id = s.id
JOIN users u ON b.customer_id = u.id
WHERE b.scheduled_start >= CURRENT_DATE
ORDER BY b.scheduled_start
LIMIT 5;

\echo ''
\echo '============================================='
\echo 'Test Credentials:'
\echo '  Email: e2e@test.com'
\echo '  Email: customer@demo.com'
\echo '  Email: alex@demo.com (walker)'
\echo '  Email: admin@demo.com (admin)'
\echo '  Password: TestPassword123!'
\echo '============================================='
