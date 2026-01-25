-- =====================================================
-- OFFLEASH Colorado Front Range Demo Data
-- =====================================================
-- Creates 50+ clients, 5 walkers with service areas, 90-day booking history
-- Run with: psql -d offleash -f demos/colorado-seed-data.sql
-- =====================================================

-- Configuration
\set org_id '7af5b534-738f-45e1-829b-64689a85393f'
\set password_hash '$argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpamtsbW5vcA$rKzFQfWVTxP8ywKYjVqQm8JgBB8qW3OhYKMLvQCzYxQ'

-- =====================================================
-- Step 1: Ensure organization exists
-- =====================================================
INSERT INTO organizations (id, name, slug, subdomain, created_at, updated_at)
VALUES (
    :'org_id'::uuid,
    'OFFLEASH Colorado',
    'offleash-colorado',
    'colorado',
    NOW(),
    NOW()
)
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;

-- =====================================================
-- Step 2: Create 5 Walker Accounts
-- =====================================================

-- Walker 1: Alex - Downtown Denver / RiNo / LoHi / Highland
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'a01c0000-0000-0000-0000-000000000001'::uuid,
    'alex.downtown@offleash.co',
    :'password_hash',
    'walker',
    'Alex',
    'Martinez',
    '303-555-0101',
    :'org_id'::uuid,
    NOW() - INTERVAL '6 months',
    NOW()
)
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, last_name = EXCLUDED.last_name;

-- Walker 2: Sarah - Cherry Creek / Washington Park / Congress Park / Cap Hill
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'a01c0000-0000-0000-0000-000000000002'::uuid,
    'sarah.cherrycreek@offleash.co',
    :'password_hash',
    'walker',
    'Sarah',
    'Johnson',
    '303-555-0102',
    :'org_id'::uuid,
    NOW() - INTERVAL '8 months',
    NOW()
)
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, last_name = EXCLUDED.last_name;

-- Walker 3: Mike - Boulder / Louisville / Superior / Broomfield
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'a01c0000-0000-0000-0000-000000000003'::uuid,
    'mike.boulder@offleash.co',
    :'password_hash',
    'walker',
    'Mike',
    'Chen',
    '303-555-0103',
    :'org_id'::uuid,
    NOW() - INTERVAL '4 months',
    NOW()
)
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, last_name = EXCLUDED.last_name;

-- Walker 4: Jessica - Arvada / Westminster / Thornton / Northglenn
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'a01c0000-0000-0000-0000-000000000004'::uuid,
    'jessica.arvada@offleash.co',
    :'password_hash',
    'walker',
    'Jessica',
    'Williams',
    '303-555-0104',
    :'org_id'::uuid,
    NOW() - INTERVAL '5 months',
    NOW()
)
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, last_name = EXCLUDED.last_name;

-- Walker 5: David - Lakewood / Golden / Wheat Ridge / Edgewater
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES (
    'a01c0000-0000-0000-0000-000000000005'::uuid,
    'david.lakewood@offleash.co',
    :'password_hash',
    'walker',
    'David',
    'Thompson',
    '303-555-0105',
    :'org_id'::uuid,
    NOW() - INTERVAL '7 months',
    NOW()
)
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, last_name = EXCLUDED.last_name;

-- =====================================================
-- Step 3: Create Walker Profiles
-- =====================================================

INSERT INTO walker_profiles (id, user_id, organization_id, bio, years_experience, created_at, updated_at)
VALUES
    ('b01c0000-0000-0000-0000-000000000001'::uuid, 'a01c0000-0000-0000-0000-000000000001'::uuid, :'org_id'::uuid,
     'Denver native specializing in high-energy breeds. Love exploring RiNo and LoHi neighborhoods!', 5, NOW(), NOW()),
    ('b01c0000-0000-0000-0000-000000000002'::uuid, 'a01c0000-0000-0000-0000-000000000002'::uuid, :'org_id'::uuid,
     'Certified in pet first aid. Washington Park is my favorite walking spot!', 7, NOW(), NOW()),
    ('b01c0000-0000-0000-0000-000000000003'::uuid, 'a01c0000-0000-0000-0000-000000000003'::uuid, :'org_id'::uuid,
     'Boulder local with a passion for trail walks. Great with anxious dogs.', 3, NOW(), NOW()),
    ('b01c0000-0000-0000-0000-000000000004'::uuid, 'a01c0000-0000-0000-0000-000000000004'::uuid, :'org_id'::uuid,
     'Experienced with large breeds and multiple dog households. Arvada area expert.', 6, NOW(), NOW()),
    ('b01c0000-0000-0000-0000-000000000005'::uuid, 'a01c0000-0000-0000-0000-000000000005'::uuid, :'org_id'::uuid,
     'Golden and Lakewood specialist. Love hiking Clear Creek Trail with dogs!', 4, NOW(), NOW())
ON CONFLICT (user_id, organization_id) DO UPDATE SET bio = EXCLUDED.bio;

-- =====================================================
-- Step 3b: Create User Identities for Email Login
-- =====================================================

INSERT INTO user_identities (user_id, provider, provider_user_id, provider_email)
VALUES
    ('a01c0000-0000-0000-0000-000000000001'::uuid, 'email', 'alex.downtown@offleash.co', 'alex.downtown@offleash.co'),
    ('a01c0000-0000-0000-0000-000000000002'::uuid, 'email', 'sarah.cherrycreek@offleash.co', 'sarah.cherrycreek@offleash.co'),
    ('a01c0000-0000-0000-0000-000000000003'::uuid, 'email', 'mike.boulder@offleash.co', 'mike.boulder@offleash.co'),
    ('a01c0000-0000-0000-0000-000000000004'::uuid, 'email', 'jessica.arvada@offleash.co', 'jessica.arvada@offleash.co'),
    ('a01c0000-0000-0000-0000-000000000005'::uuid, 'email', 'david.lakewood@offleash.co', 'david.lakewood@offleash.co')
ON CONFLICT (provider, provider_user_id) DO NOTHING;

-- =====================================================
-- Step 3c: Create Memberships for Walkers
-- =====================================================

INSERT INTO memberships (user_id, organization_id, role, status, joined_at)
VALUES
    ('a01c0000-0000-0000-0000-000000000001'::uuid, :'org_id'::uuid, 'walker', 'active', NOW() - INTERVAL '6 months'),
    ('a01c0000-0000-0000-0000-000000000002'::uuid, :'org_id'::uuid, 'walker', 'active', NOW() - INTERVAL '8 months'),
    ('a01c0000-0000-0000-0000-000000000003'::uuid, :'org_id'::uuid, 'walker', 'active', NOW() - INTERVAL '4 months'),
    ('a01c0000-0000-0000-0000-000000000004'::uuid, :'org_id'::uuid, 'walker', 'active', NOW() - INTERVAL '5 months'),
    ('a01c0000-0000-0000-0000-000000000005'::uuid, :'org_id'::uuid, 'walker', 'active', NOW() - INTERVAL '7 months')
ON CONFLICT (user_id, organization_id, role) DO NOTHING;

-- =====================================================
-- Step 4: Create Walker Service Areas (Polygons)
-- =====================================================

-- Walker 1: Downtown Denver / RiNo / LoHi / Highland
INSERT INTO service_areas (id, organization_id, walker_id, name, color, polygon, is_active)
VALUES (
    'd01c0000-0000-0000-0000-000000000001'::uuid,
    :'org_id'::uuid,
    'a01c0000-0000-0000-0000-000000000001'::uuid,
    'Downtown Denver Area',
    '#EF4444',
    '[
        {"lat": 39.7800, "lng": -105.0300},
        {"lat": 39.7800, "lng": -104.9600},
        {"lat": 39.7300, "lng": -104.9600},
        {"lat": 39.7300, "lng": -105.0300}
    ]'::jsonb,
    true
)
ON CONFLICT (id) DO UPDATE SET polygon = EXCLUDED.polygon;

-- Walker 2: Cherry Creek / Washington Park / Congress Park / Cap Hill
INSERT INTO service_areas (id, organization_id, walker_id, name, color, polygon, is_active)
VALUES (
    'd01c0000-0000-0000-0000-000000000002'::uuid,
    :'org_id'::uuid,
    'a01c0000-0000-0000-0000-000000000002'::uuid,
    'Cherry Creek & South Denver',
    '#22C55E',
    '[
        {"lat": 39.7400, "lng": -104.9900},
        {"lat": 39.7400, "lng": -104.9200},
        {"lat": 39.6800, "lng": -104.9200},
        {"lat": 39.6800, "lng": -104.9900}
    ]'::jsonb,
    true
)
ON CONFLICT (id) DO UPDATE SET polygon = EXCLUDED.polygon;

-- Walker 3: Boulder / Louisville / Superior / Broomfield
INSERT INTO service_areas (id, organization_id, walker_id, name, color, polygon, is_active)
VALUES (
    'd01c0000-0000-0000-0000-000000000003'::uuid,
    :'org_id'::uuid,
    'a01c0000-0000-0000-0000-000000000003'::uuid,
    'Boulder County',
    '#3B82F6',
    '[
        {"lat": 40.0500, "lng": -105.3000},
        {"lat": 40.0500, "lng": -105.0800},
        {"lat": 39.9300, "lng": -105.0800},
        {"lat": 39.9300, "lng": -105.3000}
    ]'::jsonb,
    true
)
ON CONFLICT (id) DO UPDATE SET polygon = EXCLUDED.polygon;

-- Walker 4: Arvada / Westminster / Thornton / Northglenn
INSERT INTO service_areas (id, organization_id, walker_id, name, color, polygon, is_active)
VALUES (
    'd01c0000-0000-0000-0000-000000000004'::uuid,
    :'org_id'::uuid,
    'a01c0000-0000-0000-0000-000000000004'::uuid,
    'North Metro Denver',
    '#A855F7',
    '[
        {"lat": 39.9300, "lng": -105.1200},
        {"lat": 39.9300, "lng": -104.9500},
        {"lat": 39.8000, "lng": -104.9500},
        {"lat": 39.8000, "lng": -105.1200}
    ]'::jsonb,
    true
)
ON CONFLICT (id) DO UPDATE SET polygon = EXCLUDED.polygon;

-- Walker 5: Lakewood / Golden / Wheat Ridge / Edgewater
INSERT INTO service_areas (id, organization_id, walker_id, name, color, polygon, is_active)
VALUES (
    'd01c0000-0000-0000-0000-000000000005'::uuid,
    :'org_id'::uuid,
    'a01c0000-0000-0000-0000-000000000005'::uuid,
    'West Denver & Foothills',
    '#F59E0B',
    '[
        {"lat": 39.8000, "lng": -105.2500},
        {"lat": 39.8000, "lng": -105.0600},
        {"lat": 39.6800, "lng": -105.0600},
        {"lat": 39.6800, "lng": -105.2500}
    ]'::jsonb,
    true
)
ON CONFLICT (id) DO UPDATE SET polygon = EXCLUDED.polygon;

-- =====================================================
-- Step 5: Create Walker Working Hours (Mon-Fri 8am-6pm, Sat 9am-3pm)
-- =====================================================

INSERT INTO working_hours (id, walker_id, organization_id, day_of_week, start_time, end_time, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(),
    w.walker_id,
    :'org_id'::uuid,
    d.dow,
    CASE WHEN d.dow = 6 THEN '09:00:00'::time ELSE '08:00:00'::time END,
    CASE WHEN d.dow = 6 THEN '15:00:00'::time ELSE '18:00:00'::time END,
    d.dow != 0,  -- Sunday off
    NOW(),
    NOW()
FROM (VALUES
    ('a01c0000-0000-0000-0000-000000000001'::uuid),
    ('a01c0000-0000-0000-0000-000000000002'::uuid),
    ('a01c0000-0000-0000-0000-000000000003'::uuid),
    ('a01c0000-0000-0000-0000-000000000004'::uuid),
    ('a01c0000-0000-0000-0000-000000000005'::uuid)
) AS w(walker_id)
CROSS JOIN generate_series(0, 6) AS d(dow)
ON CONFLICT DO NOTHING;

-- =====================================================
-- Step 6: Create 50+ Customer Accounts with Colorado Addresses
-- =====================================================

-- Downtown Denver / RiNo / LoHi Customers (Walker 1's area)
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES
    ('c01c0000-0000-0000-0000-000000000001'::uuid, 'emma.california@demo.com', :'password_hash', 'customer', 'Emma', 'Rodriguez', '303-555-1001', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000002'::uuid, 'james.larimer@demo.com', :'password_hash', 'customer', 'James', 'Wilson', '303-555-1002', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000003'::uuid, 'sophia.tejon@demo.com', :'password_hash', 'customer', 'Sophia', 'Garcia', '303-555-1003', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000004'::uuid, 'oliver.boulder@demo.com', :'password_hash', 'customer', 'Oliver', 'Brown', '303-555-1004', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000005'::uuid, 'mia.wynkoop@demo.com', :'password_hash', 'customer', 'Mia', 'Davis', '303-555-1005', :'org_id'::uuid, NOW() - INTERVAL '1 month', NOW()),
    ('c01c0000-0000-0000-0000-000000000006'::uuid, 'liam.champa@demo.com', :'password_hash', 'customer', 'Liam', 'Martinez', '303-555-1006', :'org_id'::uuid, NOW() - INTERVAL '6 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000007'::uuid, 'ava.walnut@demo.com', :'password_hash', 'customer', 'Ava', 'Anderson', '303-555-1007', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000008'::uuid, 'noah.blake@demo.com', :'password_hash', 'customer', 'Noah', 'Taylor', '303-555-1008', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000009'::uuid, 'isabella.highland@demo.com', :'password_hash', 'customer', 'Isabella', 'Thomas', '303-555-1009', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000010'::uuid, 'ethan.market@demo.com', :'password_hash', 'customer', 'Ethan', 'Jackson', '303-555-1010', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW())
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name;

-- Cherry Creek / Washington Park / Congress Park / Cap Hill Customers (Walker 2's area)
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES
    ('c01c0000-0000-0000-0000-000000000011'::uuid, 'amelia.cherrycreek@demo.com', :'password_hash', 'customer', 'Amelia', 'White', '303-555-1011', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000012'::uuid, 'benjamin.washpark@demo.com', :'password_hash', 'customer', 'Benjamin', 'Harris', '303-555-1012', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000013'::uuid, 'charlotte.logan@demo.com', :'password_hash', 'customer', 'Charlotte', 'Martin', '303-555-1013', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000014'::uuid, 'henry.detroit@demo.com', :'password_hash', 'customer', 'Henry', 'Thompson', '303-555-1014', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000015'::uuid, 'luna.colfax@demo.com', :'password_hash', 'customer', 'Luna', 'Garcia', '303-555-1015', :'org_id'::uuid, NOW() - INTERVAL '6 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000016'::uuid, 'jack.downing@demo.com', :'password_hash', 'customer', 'Jack', 'Martinez', '303-555-1016', :'org_id'::uuid, NOW() - INTERVAL '1 month', NOW()),
    ('c01c0000-0000-0000-0000-000000000017'::uuid, 'harper.josephine@demo.com', :'password_hash', 'customer', 'Harper', 'Robinson', '303-555-1017', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000018'::uuid, 'lucas.york@demo.com', :'password_hash', 'customer', 'Lucas', 'Clark', '303-555-1018', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000019'::uuid, 'evelyn.gaylord@demo.com', :'password_hash', 'customer', 'Evelyn', 'Rodriguez', '303-555-1019', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000020'::uuid, 'mason.franklin@demo.com', :'password_hash', 'customer', 'Mason', 'Lewis', '303-555-1020', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW())
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name;

-- Boulder / Louisville / Superior / Broomfield Customers (Walker 3's area)
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES
    ('c01c0000-0000-0000-0000-000000000021'::uuid, 'aria.pearl@demo.com', :'password_hash', 'customer', 'Aria', 'Lee', '303-555-1021', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000022'::uuid, 'aiden.baseline@demo.com', :'password_hash', 'customer', 'Aiden', 'Walker', '303-555-1022', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000023'::uuid, 'chloe.louisville@demo.com', :'password_hash', 'customer', 'Chloe', 'Hall', '303-555-1023', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000024'::uuid, 'logan.superior@demo.com', :'password_hash', 'customer', 'Logan', 'Young', '303-555-1024', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000025'::uuid, 'zoey.arapahoe@demo.com', :'password_hash', 'customer', 'Zoey', 'Allen', '303-555-1025', :'org_id'::uuid, NOW() - INTERVAL '1 month', NOW()),
    ('c01c0000-0000-0000-0000-000000000026'::uuid, 'carter.canyon@demo.com', :'password_hash', 'customer', 'Carter', 'King', '303-555-1026', :'org_id'::uuid, NOW() - INTERVAL '6 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000027'::uuid, 'stella.broadway@demo.com', :'password_hash', 'customer', 'Stella', 'Wright', '303-555-1027', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000028'::uuid, 'jayden.spruce@demo.com', :'password_hash', 'customer', 'Jayden', 'Scott', '303-555-1028', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000029'::uuid, 'layla.mapleton@demo.com', :'password_hash', 'customer', 'Layla', 'Green', '303-555-1029', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000030'::uuid, 'ryan.28th@demo.com', :'password_hash', 'customer', 'Ryan', 'Adams', '303-555-1030', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW())
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name;

-- Arvada / Westminster / Thornton / Northglenn Customers (Walker 4's area)
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES
    ('c01c0000-0000-0000-0000-000000000031'::uuid, 'ellie.80th@demo.com', :'password_hash', 'customer', 'Ellie', 'Baker', '303-555-1031', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000032'::uuid, 'nathan.88th@demo.com', :'password_hash', 'customer', 'Nathan', 'Gonzalez', '303-555-1032', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000033'::uuid, 'lily.washington@demo.com', :'password_hash', 'customer', 'Lily', 'Nelson', '303-555-1033', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000034'::uuid, 'owen.120th@demo.com', :'password_hash', 'customer', 'Owen', 'Carter', '303-555-1034', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000035'::uuid, 'riley.sheridan@demo.com', :'password_hash', 'customer', 'Riley', 'Mitchell', '303-555-1035', :'org_id'::uuid, NOW() - INTERVAL '6 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000036'::uuid, 'hunter.wadsworth@demo.com', :'password_hash', 'customer', 'Hunter', 'Perez', '303-555-1036', :'org_id'::uuid, NOW() - INTERVAL '1 month', NOW()),
    ('c01c0000-0000-0000-0000-000000000037'::uuid, 'nora.ralston@demo.com', :'password_hash', 'customer', 'Nora', 'Roberts', '303-555-1037', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000038'::uuid, 'wyatt.federal@demo.com', :'password_hash', 'customer', 'Wyatt', 'Turner', '303-555-1038', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000039'::uuid, 'hazel.kipling@demo.com', :'password_hash', 'customer', 'Hazel', 'Phillips', '303-555-1039', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000040'::uuid, 'eli.huron@demo.com', :'password_hash', 'customer', 'Eli', 'Campbell', '303-555-1040', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW())
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name;

-- Lakewood / Golden / Wheat Ridge / Edgewater Customers (Walker 5's area)
INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, organization_id, created_at, updated_at)
VALUES
    ('c01c0000-0000-0000-0000-000000000041'::uuid, 'violet.wadsworth@demo.com', :'password_hash', 'customer', 'Violet', 'Parker', '303-555-1041', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000042'::uuid, 'luke.washingtonave@demo.com', :'password_hash', 'customer', 'Luke', 'Evans', '303-555-1042', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000043'::uuid, 'penelope.youngfield@demo.com', :'password_hash', 'customer', 'Penelope', 'Edwards', '303-555-1043', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000044'::uuid, 'gabriel.colfaxw@demo.com', :'password_hash', 'customer', 'Gabriel', 'Collins', '303-555-1044', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000045'::uuid, 'aurora.alameda@demo.com', :'password_hash', 'customer', 'Aurora', 'Stewart', '303-555-1045', :'org_id'::uuid, NOW() - INTERVAL '1 month', NOW()),
    ('c01c0000-0000-0000-0000-000000000046'::uuid, 'asher.morrison@demo.com', :'password_hash', 'customer', 'Asher', 'Sanchez', '303-555-1046', :'org_id'::uuid, NOW() - INTERVAL '6 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000047'::uuid, 'savannah.6th@demo.com', :'password_hash', 'customer', 'Savannah', 'Morris', '303-555-1047', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000048'::uuid, 'ezra.indiana@demo.com', :'password_hash', 'customer', 'Ezra', 'Rogers', '303-555-1048', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000049'::uuid, 'audrey.20th@demo.com', :'password_hash', 'customer', 'Audrey', 'Reed', '303-555-1049', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000050'::uuid, 'caleb.simms@demo.com', :'password_hash', 'customer', 'Caleb', 'Cook', '303-555-1050', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW()),
    -- Extra customers for more variety
    ('c01c0000-0000-0000-0000-000000000051'::uuid, 'grace.sloansake@demo.com', :'password_hash', 'customer', 'Grace', 'Morgan', '303-555-1051', :'org_id'::uuid, NOW() - INTERVAL '3 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000052'::uuid, 'daniel.belmar@demo.com', :'password_hash', 'customer', 'Daniel', 'Bell', '303-555-1052', :'org_id'::uuid, NOW() - INTERVAL '4 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000053'::uuid, 'brooklyn.clearcreek@demo.com', :'password_hash', 'customer', 'Brooklyn', 'Murphy', '303-555-1053', :'org_id'::uuid, NOW() - INTERVAL '2 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000054'::uuid, 'dylan.unionstation@demo.com', :'password_hash', 'customer', 'Dylan', 'Bailey', '303-555-1054', :'org_id'::uuid, NOW() - INTERVAL '5 months', NOW()),
    ('c01c0000-0000-0000-0000-000000000055'::uuid, 'madelyn.speer@demo.com', :'password_hash', 'customer', 'Madelyn', 'Rivera', '303-555-1055', :'org_id'::uuid, NOW() - INTERVAL '1 month', NOW())
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name;

-- =====================================================
-- Step 7: Create Customer Locations with Real Colorado Addresses
-- =====================================================

-- Downtown Denver / RiNo / LoHi / Highland Locations
INSERT INTO locations (id, user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at)
VALUES
    ('e01c0000-0000-0000-0000-000000000001'::uuid, 'c01c0000-0000-0000-0000-000000000001'::uuid, :'org_id'::uuid, 'Home', '1600 California St', 'Denver', 'CO', '80202', 39.7456, -104.9894, 'Downtown high-rise, buzzer 1601', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000002'::uuid, 'c01c0000-0000-0000-0000-000000000002'::uuid, :'org_id'::uuid, 'Home', '2936 Larimer St', 'Denver', 'CO', '80205', 39.7616, -104.9784, 'RiNo loft - street level entrance', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000003'::uuid, 'c01c0000-0000-0000-0000-000000000003'::uuid, :'org_id'::uuid, 'Home', '3200 Tejon St', 'Denver', 'CO', '80211', 39.7612, -105.0106, 'Highland - yellow Victorian', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000004'::uuid, 'c01c0000-0000-0000-0000-000000000004'::uuid, :'org_id'::uuid, 'Home', '1550 Boulder St', 'Denver', 'CO', '80211', 39.7584, -105.0069, 'LoHi townhouse', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000005'::uuid, 'c01c0000-0000-0000-0000-000000000005'::uuid, :'org_id'::uuid, 'Home', '1701 Wynkoop St', 'Denver', 'CO', '80202', 39.7526, -104.9996, 'Union Station area condo', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000006'::uuid, 'c01c0000-0000-0000-0000-000000000006'::uuid, :'org_id'::uuid, 'Home', '1801 Champa St', 'Denver', 'CO', '80202', 39.7489, -104.9923, 'LoDo apartment', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000007'::uuid, 'c01c0000-0000-0000-0000-000000000007'::uuid, :'org_id'::uuid, 'Home', '2700 Walnut St', 'Denver', 'CO', '80205', 39.7603, -104.9809, 'RiNo industrial conversion', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000008'::uuid, 'c01c0000-0000-0000-0000-000000000008'::uuid, :'org_id'::uuid, 'Home', '3100 Blake St', 'Denver', 'CO', '80205', 39.7639, -104.9786, 'Near Coors Field', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000009'::uuid, 'c01c0000-0000-0000-0000-000000000009'::uuid, :'org_id'::uuid, 'Home', '3401 Navajo St', 'Denver', 'CO', '80211', 39.7658, -105.0117, 'Highland Square area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000010'::uuid, 'c01c0000-0000-0000-0000-000000000010'::uuid, :'org_id'::uuid, 'Home', '1900 Market St', 'Denver', 'CO', '80202', 39.7512, -104.9912, 'Ballpark neighborhood', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET address = EXCLUDED.address;

-- Cherry Creek / Washington Park / Congress Park / Cap Hill Locations
INSERT INTO locations (id, user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at)
VALUES
    ('e01c0000-0000-0000-0000-000000000011'::uuid, 'c01c0000-0000-0000-0000-000000000011'::uuid, :'org_id'::uuid, 'Home', '2500 E 2nd Ave', 'Denver', 'CO', '80206', 39.7176, -104.9548, 'Cherry Creek North', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000012'::uuid, 'c01c0000-0000-0000-0000-000000000012'::uuid, :'org_id'::uuid, 'Home', '700 S Downing St', 'Denver', 'CO', '80209', 39.7052, -104.9728, 'Washington Park East', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000013'::uuid, 'c01c0000-0000-0000-0000-000000000013'::uuid, :'org_id'::uuid, 'Home', '1100 Logan St', 'Denver', 'CO', '80203', 39.7312, -104.9826, 'Cap Hill Victorian', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000014'::uuid, 'c01c0000-0000-0000-0000-000000000014'::uuid, :'org_id'::uuid, 'Home', '1200 Detroit St', 'Denver', 'CO', '80206', 39.7223, -104.9564, 'Congress Park bungalow', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000015'::uuid, 'c01c0000-0000-0000-0000-000000000015'::uuid, :'org_id'::uuid, 'Home', '1400 E Colfax Ave', 'Denver', 'CO', '80218', 39.7401, -104.9678, 'City Park West', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000016'::uuid, 'c01c0000-0000-0000-0000-000000000016'::uuid, :'org_id'::uuid, 'Home', '850 S Downing St', 'Denver', 'CO', '80209', 39.7028, -104.9728, 'Wash Park West', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000017'::uuid, 'c01c0000-0000-0000-0000-000000000017'::uuid, :'org_id'::uuid, 'Home', '250 Josephine St', 'Denver', 'CO', '80206', 39.7195, -104.9548, 'Cherry Creek townhome', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000018'::uuid, 'c01c0000-0000-0000-0000-000000000018'::uuid, :'org_id'::uuid, 'Home', '1301 York St', 'Denver', 'CO', '80206', 39.7345, -104.9620, 'Congress Park', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000019'::uuid, 'c01c0000-0000-0000-0000-000000000019'::uuid, :'org_id'::uuid, 'Home', '545 Gaylord St', 'Denver', 'CO', '80206', 39.7256, -104.9564, 'Country Club area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000020'::uuid, 'c01c0000-0000-0000-0000-000000000020'::uuid, :'org_id'::uuid, 'Home', '700 Franklin St', 'Denver', 'CO', '80218', 39.7345, -104.9728, 'Cap Hill north', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET address = EXCLUDED.address;

-- Boulder / Louisville / Superior / Broomfield Locations
INSERT INTO locations (id, user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at)
VALUES
    ('e01c0000-0000-0000-0000-000000000021'::uuid, 'c01c0000-0000-0000-0000-000000000021'::uuid, :'org_id'::uuid, 'Home', '1600 Pearl St', 'Boulder', 'CO', '80302', 40.0176, -105.2789, 'Pearl St Mall area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000022'::uuid, 'c01c0000-0000-0000-0000-000000000022'::uuid, :'org_id'::uuid, 'Home', '2200 Baseline Rd', 'Boulder', 'CO', '80305', 39.9908, -105.2573, 'South Boulder - Chautauqua', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000023'::uuid, 'c01c0000-0000-0000-0000-000000000023'::uuid, :'org_id'::uuid, 'Home', '700 Main St', 'Louisville', 'CO', '80027', 39.9778, -105.1319, 'Historic downtown Louisville', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000024'::uuid, 'c01c0000-0000-0000-0000-000000000024'::uuid, :'org_id'::uuid, 'Home', '301 Discovery Pkwy', 'Superior', 'CO', '80027', 39.9528, -105.1686, 'Rock Creek area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000025'::uuid, 'c01c0000-0000-0000-0000-000000000025'::uuid, :'org_id'::uuid, 'Home', '3300 Arapahoe Ave', 'Boulder', 'CO', '80303', 40.0145, -105.2456, 'East Boulder', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000026'::uuid, 'c01c0000-0000-0000-0000-000000000026'::uuid, :'org_id'::uuid, 'Home', '2850 Canyon Blvd', 'Boulder', 'CO', '80302', 40.0189, -105.2678, 'Near CU campus', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000027'::uuid, 'c01c0000-0000-0000-0000-000000000027'::uuid, :'org_id'::uuid, 'Home', '1900 Broadway', 'Boulder', 'CO', '80302', 40.0212, -105.2745, 'North Boulder', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000028'::uuid, 'c01c0000-0000-0000-0000-000000000028'::uuid, :'org_id'::uuid, 'Home', '820 Spruce St', 'Louisville', 'CO', '80027', 39.9789, -105.1356, 'Louisville near park', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000029'::uuid, 'c01c0000-0000-0000-0000-000000000029'::uuid, :'org_id'::uuid, 'Home', '1000 Mapleton Ave', 'Boulder', 'CO', '80304', 40.0234, -105.2812, 'Mapleton Hill', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000030'::uuid, 'c01c0000-0000-0000-0000-000000000030'::uuid, :'org_id'::uuid, 'Home', '2800 28th St', 'Boulder', 'CO', '80301', 40.0178, -105.2512, 'Twenty Ninth Street area', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET address = EXCLUDED.address;

-- Arvada / Westminster / Thornton / Northglenn Locations
INSERT INTO locations (id, user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at)
VALUES
    ('e01c0000-0000-0000-0000-000000000031'::uuid, 'c01c0000-0000-0000-0000-000000000031'::uuid, :'org_id'::uuid, 'Home', '7500 W 80th Ave', 'Arvada', 'CO', '80003', 39.8427, -105.0742, 'Olde Town Arvada', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000032'::uuid, 'c01c0000-0000-0000-0000-000000000032'::uuid, :'org_id'::uuid, 'Home', '8200 W 88th Ave', 'Westminster', 'CO', '80021', 39.8556, -105.0217, 'Westminster Promenade area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000033'::uuid, 'c01c0000-0000-0000-0000-000000000033'::uuid, :'org_id'::uuid, 'Home', '9551 Washington St', 'Thornton', 'CO', '80229', 39.8683, -104.9869, 'Near I-25', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000034'::uuid, 'c01c0000-0000-0000-0000-000000000034'::uuid, :'org_id'::uuid, 'Home', '4900 W 120th Ave', 'Broomfield', 'CO', '80020', 39.9273, -105.0253, 'FlatIron Crossing area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000035'::uuid, 'c01c0000-0000-0000-0000-000000000035'::uuid, :'org_id'::uuid, 'Home', '6700 Sheridan Blvd', 'Arvada', 'CO', '80003', 39.8312, -105.0539, 'Arvada West', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000036'::uuid, 'c01c0000-0000-0000-0000-000000000036'::uuid, :'org_id'::uuid, 'Home', '7200 N Wadsworth Blvd', 'Westminster', 'CO', '80021', 39.8478, -105.0809, 'Near Standley Lake', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000037'::uuid, 'c01c0000-0000-0000-0000-000000000037'::uuid, :'org_id'::uuid, 'Home', '10301 Ralston Rd', 'Arvada', 'CO', '80004', 39.8134, -105.1256, 'West Arvada', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000038'::uuid, 'c01c0000-0000-0000-0000-000000000038'::uuid, :'org_id'::uuid, 'Home', '8400 Federal Blvd', 'Westminster', 'CO', '80031', 39.8567, -105.0242, 'Westminster central', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000039'::uuid, 'c01c0000-0000-0000-0000-000000000039'::uuid, :'org_id'::uuid, 'Home', '11601 N Kipling St', 'Westminster', 'CO', '80021', 39.8934, -105.0809, 'Northwest Westminster', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000040'::uuid, 'c01c0000-0000-0000-0000-000000000040'::uuid, :'org_id'::uuid, 'Home', '10600 Huron St', 'Northglenn', 'CO', '80234', 39.8845, -104.9978, 'Northglenn', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET address = EXCLUDED.address;

-- Lakewood / Golden / Wheat Ridge / Edgewater Locations
INSERT INTO locations (id, user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at)
VALUES
    ('e01c0000-0000-0000-0000-000000000041'::uuid, 'c01c0000-0000-0000-0000-000000000041'::uuid, :'org_id'::uuid, 'Home', '1600 S Wadsworth Blvd', 'Lakewood', 'CO', '80232', 39.6939, -105.0809, 'Villa Italia area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000042'::uuid, 'c01c0000-0000-0000-0000-000000000042'::uuid, :'org_id'::uuid, 'Home', '1200 Washington Ave', 'Golden', 'CO', '80401', 39.7555, -105.2211, 'Downtown Golden', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000043'::uuid, 'c01c0000-0000-0000-0000-000000000043'::uuid, :'org_id'::uuid, 'Home', '2400 Youngfield St', 'Lakewood', 'CO', '80215', 39.7436, -105.1178, 'Applewood area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000044'::uuid, 'c01c0000-0000-0000-0000-000000000044'::uuid, :'org_id'::uuid, 'Home', '10500 W Colfax Ave', 'Lakewood', 'CO', '80215', 39.7401, -105.1312, 'West Colfax', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000045'::uuid, 'c01c0000-0000-0000-0000-000000000045'::uuid, :'org_id'::uuid, 'Home', '700 W Alameda Ave', 'Lakewood', 'CO', '80226', 39.7134, -105.0312, 'Mar Lee', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000046'::uuid, 'c01c0000-0000-0000-0000-000000000046'::uuid, :'org_id'::uuid, 'Home', '16900 W Morrison Rd', 'Morrison', 'CO', '80465', 39.6534, -105.1923, 'Near Red Rocks', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000047'::uuid, 'c01c0000-0000-0000-0000-000000000047'::uuid, :'org_id'::uuid, 'Home', '500 6th Ave', 'Golden', 'CO', '80401', 39.7556, -105.2189, 'Near School of Mines', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000048'::uuid, 'c01c0000-0000-0000-0000-000000000048'::uuid, :'org_id'::uuid, 'Home', '4300 Indiana St', 'Golden', 'CO', '80403', 39.7623, -105.1856, 'Pleasant View', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000049'::uuid, 'c01c0000-0000-0000-0000-000000000049'::uuid, :'org_id'::uuid, 'Home', '5600 W 20th Ave', 'Edgewater', 'CO', '80214', 39.7523, -105.0689, 'Edgewater', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000050'::uuid, 'c01c0000-0000-0000-0000-000000000050'::uuid, :'org_id'::uuid, 'Home', '4300 Simms St', 'Wheat Ridge', 'CO', '80033', 39.7756, -105.1423, 'Wheat Ridge Greenbelt', true, NOW(), NOW()),
    -- Extra locations
    ('e01c0000-0000-0000-0000-000000000051'::uuid, 'c01c0000-0000-0000-0000-000000000051'::uuid, :'org_id'::uuid, 'Home', '2600 S Sloans Lake Dr', 'Denver', 'CO', '80212', 39.7423, -105.0423, 'Sloans Lake', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000052'::uuid, 'c01c0000-0000-0000-0000-000000000052'::uuid, :'org_id'::uuid, 'Home', '1800 Belmar Ln', 'Lakewood', 'CO', '80226', 39.7012, -105.0745, 'Belmar area', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000053'::uuid, 'c01c0000-0000-0000-0000-000000000053'::uuid, :'org_id'::uuid, 'Home', '14000 W Colfax Ave', 'Golden', 'CO', '80401', 39.7398, -105.1789, 'Golden/Lakewood border', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000054'::uuid, 'c01c0000-0000-0000-0000-000000000054'::uuid, :'org_id'::uuid, 'Home', '1600 17th St', 'Denver', 'CO', '80202', 39.7512, -104.9956, 'Union Station', true, NOW(), NOW()),
    ('e01c0000-0000-0000-0000-000000000055'::uuid, 'c01c0000-0000-0000-0000-000000000055'::uuid, :'org_id'::uuid, 'Home', '400 E Speer Blvd', 'Denver', 'CO', '80203', 39.7234, -104.9867, 'Speer/Broadway', true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET address = EXCLUDED.address;

-- =====================================================
-- Step 8: Create Services
-- =====================================================

INSERT INTO services (id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at)
VALUES
    ('f01c0000-0000-0000-0000-000000000001'::uuid, :'org_id'::uuid, '30 Min Walk', 'A refreshing 30-minute neighborhood walk', 30, 2500, true, NOW(), NOW()),
    ('f01c0000-0000-0000-0000-000000000002'::uuid, :'org_id'::uuid, '60 Min Adventure Walk', 'A full hour of exploration and exercise', 60, 4000, true, NOW(), NOW()),
    ('f01c0000-0000-0000-0000-000000000003'::uuid, :'org_id'::uuid, 'Puppy Play Session', '45 minutes of supervised play for puppies', 45, 3500, true, NOW(), NOW()),
    ('f01c0000-0000-0000-0000-000000000004'::uuid, :'org_id'::uuid, 'Drop-In Visit', '15-minute check-in for feeding and potty', 15, 1500, true, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;

-- =====================================================
-- Step 9: Generate 90-Day Booking History
-- =====================================================
-- Distribution:
-- Past 30 days: 70% completed, 15% cancelled, 15% no_show
-- Today: 30% in_progress, 50% confirmed, 20% pending
-- Future 60 days: 60% confirmed, 30% pending, 10% cancelled

-- Create a function to generate bookings
DO $$
DECLARE
    v_org_id UUID := '7af5b534-738f-45e1-829b-64689a85393f'::uuid;
    v_walker_ids UUID[] := ARRAY[
        'a01c0000-0000-0000-0000-000000000001'::uuid,
        'a01c0000-0000-0000-0000-000000000002'::uuid,
        'a01c0000-0000-0000-0000-000000000003'::uuid,
        'a01c0000-0000-0000-0000-000000000004'::uuid,
        'a01c0000-0000-0000-0000-000000000005'::uuid
    ];
    v_service_ids UUID[] := ARRAY[
        'f01c0000-0000-0000-0000-000000000001'::uuid,
        'f01c0000-0000-0000-0000-000000000002'::uuid,
        'f01c0000-0000-0000-0000-000000000003'::uuid,
        'f01c0000-0000-0000-0000-000000000004'::uuid
    ];
    v_service_durations INT[] := ARRAY[30, 60, 45, 15];
    v_service_prices INT[] := ARRAY[2500, 4000, 3500, 1500];
    v_day_offset INT;
    v_walker_idx INT;
    v_customer_idx INT;
    v_service_idx INT;
    v_walker_id UUID;
    v_customer_id UUID;
    v_location_id UUID;
    v_service_id UUID;
    v_hour INT;
    v_booking_date TIMESTAMP;
    v_status TEXT;
    v_random FLOAT;
    v_duration INT;
    v_price INT;
BEGIN
    -- Loop through 90 days (-30 to +60)
    FOR v_day_offset IN -30..60 LOOP
        -- Skip Sundays
        IF EXTRACT(DOW FROM CURRENT_DATE + v_day_offset) = 0 THEN
            CONTINUE;
        END IF;

        -- Generate 3-6 bookings per walker per day
        FOR v_walker_idx IN 1..5 LOOP
            v_walker_id := v_walker_ids[v_walker_idx];

            -- Determine customers for this walker based on service area
            -- Walker 1: customers 1-10, Walker 2: 11-20, etc.
            FOR v_booking_num IN 1..(3 + floor(random() * 4)::int) LOOP
                -- Pick a random customer from this walker's area
                v_customer_idx := ((v_walker_idx - 1) * 10) + 1 + floor(random() * 10)::int;
                IF v_customer_idx > 55 THEN v_customer_idx := 55; END IF;

                v_customer_id := ('c01c0000-0000-0000-0000-0000000000' || LPAD(v_customer_idx::text, 2, '0'))::uuid;
                v_location_id := ('e01c0000-0000-0000-0000-0000000000' || LPAD(v_customer_idx::text, 2, '0'))::uuid;

                -- Pick a random service
                v_service_idx := 1 + floor(random() * 4)::int;
                v_service_id := v_service_ids[v_service_idx];
                v_duration := v_service_durations[v_service_idx];
                v_price := v_service_prices[v_service_idx];

                -- Pick a random hour (8am to 5pm, avoiding overlaps)
                v_hour := 8 + (v_booking_num * 2) + floor(random() * 2)::int;
                IF v_hour > 17 THEN v_hour := 17; END IF;

                v_booking_date := (CURRENT_DATE + v_day_offset)::timestamp + (v_hour || ' hours')::interval;

                -- Determine status based on day
                v_random := random();
                IF v_day_offset < 0 THEN
                    -- Past: 70% completed, 15% cancelled, 15% no_show
                    IF v_random < 0.70 THEN
                        v_status := 'completed';
                    ELSIF v_random < 0.85 THEN
                        v_status := 'cancelled';
                    ELSE
                        v_status := 'no_show';
                    END IF;
                ELSIF v_day_offset = 0 THEN
                    -- Today: 30% in_progress, 50% confirmed, 20% pending
                    IF v_random < 0.30 AND v_hour <= EXTRACT(HOUR FROM NOW()) THEN
                        v_status := 'in_progress';
                    ELSIF v_random < 0.80 THEN
                        v_status := 'confirmed';
                    ELSE
                        v_status := 'pending';
                    END IF;
                ELSE
                    -- Future: 60% confirmed, 30% pending, 10% cancelled
                    IF v_random < 0.60 THEN
                        v_status := 'confirmed';
                    ELSIF v_random < 0.90 THEN
                        v_status := 'pending';
                    ELSE
                        v_status := 'cancelled';
                    END IF;
                END IF;

                -- Insert booking
                INSERT INTO bookings (
                    id, customer_id, walker_id, service_id, location_id, organization_id,
                    status, scheduled_start, scheduled_end, price_cents, notes,
                    actual_start, actual_end, created_at, updated_at
                )
                VALUES (
                    gen_random_uuid(),
                    v_customer_id,
                    v_walker_id,
                    v_service_id,
                    v_location_id,
                    v_org_id,
                    v_status::booking_status,
                    v_booking_date,
                    v_booking_date + (v_duration || ' minutes')::interval,
                    v_price,
                    CASE
                        WHEN random() < 0.3 THEN 'Please ring the doorbell twice'
                        WHEN random() < 0.5 THEN 'Dog is friendly but nervous at first'
                        ELSE NULL
                    END,
                    CASE WHEN v_status IN ('completed', 'in_progress') THEN v_booking_date ELSE NULL END,
                    CASE WHEN v_status = 'completed' THEN v_booking_date + (v_duration || ' minutes')::interval ELSE NULL END,
                    v_booking_date - INTERVAL '3 days',
                    NOW()
                )
                ON CONFLICT DO NOTHING;
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

-- =====================================================
-- Step 10: Populate Travel Time Cache
-- =====================================================

-- Create travel times between common location pairs
-- This uses Haversine formula for distance calculation (no earthdistance extension needed)

INSERT INTO travel_time_cache (origin_location_id, destination_location_id, travel_seconds, distance_meters, calculated_at)
SELECT
    o.id as origin,
    d.id as destination,
    -- Calculate approximate travel time based on distance (avg 25 mph / 40 km/h in city)
    GREATEST(300, (
        -- Haversine distance in meters
        6371000 * 2 * ASIN(SQRT(
            POWER(SIN(RADIANS(d.latitude - o.latitude) / 2), 2) +
            COS(RADIANS(o.latitude)) * COS(RADIANS(d.latitude)) *
            POWER(SIN(RADIANS(d.longitude - o.longitude) / 2), 2)
        ))
        / 1000 * 2.5 * 60
    )::int) as travel_seconds,
    -- Haversine distance in meters
    (6371000 * 2 * ASIN(SQRT(
        POWER(SIN(RADIANS(d.latitude - o.latitude) / 2), 2) +
        COS(RADIANS(o.latitude)) * COS(RADIANS(d.latitude)) *
        POWER(SIN(RADIANS(d.longitude - o.longitude) / 2), 2)
    )))::int as distance_meters,
    NOW()
FROM locations o
CROSS JOIN locations d
WHERE o.id != d.id
AND o.organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
AND d.organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
ON CONFLICT (origin_location_id, destination_location_id)
DO UPDATE SET travel_seconds = EXCLUDED.travel_seconds, distance_meters = EXCLUDED.distance_meters, calculated_at = NOW();

-- =====================================================
-- Verification Queries
-- =====================================================

\echo ''
\echo '============================================='
\echo 'Colorado Demo Data Seeding Complete!'
\echo '============================================='
\echo ''

\echo '=== Walkers Created ==='
SELECT u.first_name || ' ' || u.last_name as name, u.email, sa.name as service_area
FROM users u
JOIN service_areas sa ON sa.walker_id = u.id
WHERE u.organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
AND u.role = 'walker'
ORDER BY u.first_name;

\echo ''
\echo '=== Customer Count ==='
SELECT COUNT(*) as total_customers
FROM users
WHERE organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
AND role = 'customer';

\echo ''
\echo '=== Booking Summary (Last 30 Days) ==='
SELECT status, COUNT(*) as count
FROM bookings
WHERE organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
AND scheduled_start >= CURRENT_DATE - 30
AND scheduled_start < CURRENT_DATE
GROUP BY status
ORDER BY count DESC;

\echo ''
\echo '=== Booking Summary (Today) ==='
SELECT status, COUNT(*) as count
FROM bookings
WHERE organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
AND DATE(scheduled_start) = CURRENT_DATE
GROUP BY status
ORDER BY count DESC;

\echo ''
\echo '=== Booking Summary (Next 60 Days) ==='
SELECT status, COUNT(*) as count
FROM bookings
WHERE organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
AND scheduled_start > CURRENT_DATE
AND scheduled_start <= CURRENT_DATE + 60
GROUP BY status
ORDER BY count DESC;

\echo ''
\echo '=== Total Bookings by Walker ==='
SELECT u.first_name || ' ' || u.last_name as walker, COUNT(b.id) as total_bookings
FROM users u
LEFT JOIN bookings b ON b.walker_id = u.id
WHERE u.organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
AND u.role = 'walker'
GROUP BY u.id, u.first_name, u.last_name
ORDER BY total_bookings DESC;

\echo ''
\echo '=== Travel Time Cache Entries ==='
SELECT COUNT(*) as cached_routes FROM travel_time_cache
WHERE origin_location_id IN (
    SELECT id FROM locations WHERE organization_id = '7af5b534-738f-45e1-829b-64689a85393f'::uuid
);

\echo ''
\echo '============================================='
\echo 'Test Credentials (all accounts):'
\echo '  Password: TestPassword123!'
\echo ''
\echo 'Walker Emails:'
\echo '  alex.downtown@offleash.co (Downtown/RiNo/LoHi)'
\echo '  sarah.cherrycreek@offleash.co (Cherry Creek/Wash Park)'
\echo '  mike.boulder@offleash.co (Boulder/Louisville)'
\echo '  jessica.arvada@offleash.co (Arvada/Westminster)'
\echo '  david.lakewood@offleash.co (Lakewood/Golden)'
\echo '============================================='
