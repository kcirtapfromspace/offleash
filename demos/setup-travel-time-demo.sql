-- Travel Time Demo Setup
-- Creates test customers with locations across Denver and pre-schedules bookings

-- Step 1: Create Test Customers
INSERT INTO users (email, password_hash, role, first_name, last_name, phone, organization_id)
VALUES
    ('sarah.downtown@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$abcd1234$placeholder', 'customer', 'Sarah', 'Downtown', '303-555-0101', '7af5b534-738f-45e1-829b-64689a85393f'),
    ('mike.cherrycreek@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$abcd1234$placeholder', 'customer', 'Mike', 'CherryCreek', '303-555-0102', '7af5b534-738f-45e1-829b-64689a85393f'),
    ('jen.parkhill@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$abcd1234$placeholder', 'customer', 'Jennifer', 'ParkHill', '303-555-0103', '7af5b534-738f-45e1-829b-64689a85393f'),
    ('tom.highlands@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$abcd1234$placeholder', 'customer', 'Tom', 'Highlands', '303-555-0104', '7af5b534-738f-45e1-829b-64689a85393f'),
    ('lisa.aurora@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$abcd1234$placeholder', 'customer', 'Lisa', 'Aurora', '303-555-0105', '7af5b534-738f-45e1-829b-64689a85393f')
ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name;

-- Step 2: Create Locations for Each Customer
-- Downtown Denver (Union Station area)
INSERT INTO locations (user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default)
SELECT u.id, '7af5b534-738f-45e1-829b-64689a85393f', 'Home', '1701 Wynkoop St', 'Denver', 'CO', '80202', 39.7526, -104.9996, 'Downtown loft, buzzer #201', true
FROM users u WHERE u.email = 'sarah.downtown@demo.com'
ON CONFLICT DO NOTHING;

-- Cherry Creek (~3 miles from downtown)
INSERT INTO locations (user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default)
SELECT u.id, '7af5b534-738f-45e1-829b-64689a85393f', 'Home', '250 Josephine St', 'Denver', 'CO', '80206', 39.7176, -104.9548, 'Yellow house with blue door', true
FROM users u WHERE u.email = 'mike.cherrycreek@demo.com'
ON CONFLICT DO NOTHING;

-- Park Hill (~5 miles from downtown, east)
INSERT INTO locations (user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default)
SELECT u.id, '7af5b534-738f-45e1-829b-64689a85393f', 'Home', '2301 Dahlia St', 'Denver', 'CO', '80207', 39.7517, -104.9217, 'Corner house with dog run', true
FROM users u WHERE u.email = 'jen.parkhill@demo.com'
ON CONFLICT DO NOTHING;

-- Highlands (~2 miles northwest of downtown)
INSERT INTO locations (user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default)
SELECT u.id, '7af5b534-738f-45e1-829b-64689a85393f', 'Home', '3401 Tejon St', 'Denver', 'CO', '80211', 39.7623, -105.0169, 'Historic Victorian, ring doorbell', true
FROM users u WHERE u.email = 'tom.highlands@demo.com'
ON CONFLICT DO NOTHING;

-- Aurora (~12 miles east - ACROSS TOWN)
INSERT INTO locations (user_id, organization_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default)
SELECT u.id, '7af5b534-738f-45e1-829b-64689a85393f', 'Home', '15001 E Alameda Pkwy', 'Aurora', 'CO', '80012', 39.7294, -104.8319, 'Apartment complex, building C', true
FROM users u WHERE u.email = 'lisa.aurora@demo.com'
ON CONFLICT DO NOTHING;

-- Step 3: Delete existing bookings for the walker on tomorrow
DELETE FROM bookings
WHERE walker_id = 'b376c762-b772-4fde-963e-5dcaedd52626'
AND DATE(scheduled_start) = CURRENT_DATE + 1;

-- Step 4: Create Bookings for Tomorrow
-- Uses a CTE to get all the IDs we need

WITH customer_locations AS (
    SELECT
        u.id as customer_id,
        u.email,
        l.id as location_id
    FROM users u
    JOIN locations l ON l.user_id = u.id
    WHERE u.email IN ('sarah.downtown@demo.com', 'mike.cherrycreek@demo.com', 'jen.parkhill@demo.com', 'tom.highlands@demo.com', 'lisa.aurora@demo.com')
)
INSERT INTO bookings (customer_id, walker_id, service_id, location_id, organization_id, status, scheduled_start, scheduled_end, price_cents, notes)
SELECT
    cl.customer_id,
    'b376c762-b772-4fde-963e-5dcaedd52626'::uuid,
    '319d40cd-1730-43f2-b115-200821e76856'::uuid,
    cl.location_id,
    '7af5b534-738f-45e1-829b-64689a85393f'::uuid,
    'confirmed',
    (CURRENT_DATE + 1) + b.start_time,
    (CURRENT_DATE + 1) + b.end_time,
    2500,
    b.notes
FROM (
    VALUES
        ('sarah.downtown@demo.com', TIME '09:00:00', TIME '09:30:00', 'Morning walk for Max'),
        ('mike.cherrycreek@demo.com', TIME '10:00:00', TIME '10:30:00', 'Walk for Bella - Cherry Creek'),
        ('jen.parkhill@demo.com', TIME '11:00:00', TIME '11:30:00', 'Walk for Cooper - Park Hill'),
        ('tom.highlands@demo.com', TIME '11:45:00', TIME '12:15:00', 'Walk for Luna - TIGHT from Park Hill!'),
        ('lisa.aurora@demo.com', TIME '14:00:00', TIME '14:30:00', 'Walk for Rocky - Aurora (far)'),
        ('sarah.downtown@demo.com', TIME '15:00:00', TIME '15:30:00', 'Evening walk for Max - TIGHT from Aurora!')
) AS b(email, start_time, end_time, notes)
JOIN customer_locations cl ON cl.email = b.email
ON CONFLICT DO NOTHING;

-- Step 5: Populate Travel Time Cache
-- Get location IDs and insert travel times

WITH locs AS (
    SELECT
        l.id,
        u.email,
        CASE
            WHEN u.email = 'sarah.downtown@demo.com' THEN 'downtown'
            WHEN u.email = 'mike.cherrycreek@demo.com' THEN 'cherrycreek'
            WHEN u.email = 'jen.parkhill@demo.com' THEN 'parkhill'
            WHEN u.email = 'tom.highlands@demo.com' THEN 'highlands'
            WHEN u.email = 'lisa.aurora@demo.com' THEN 'aurora'
        END as loc_name
    FROM locations l
    JOIN users u ON l.user_id = u.id
    WHERE u.email IN ('sarah.downtown@demo.com', 'mike.cherrycreek@demo.com', 'jen.parkhill@demo.com', 'tom.highlands@demo.com', 'lisa.aurora@demo.com')
)
INSERT INTO travel_time_cache (origin_location_id, destination_location_id, travel_seconds, distance_meters)
SELECT
    o.id,
    d.id,
    t.travel_seconds,
    t.travel_meters
FROM (
    -- Travel times between locations (realistic Denver estimates)
    VALUES
        -- Downtown <-> Cherry Creek: ~15 min, 5 km
        ('downtown', 'cherrycreek', 900, 5000),
        ('cherrycreek', 'downtown', 900, 5000),
        -- Cherry Creek <-> Park Hill: ~12 min, 4 km
        ('cherrycreek', 'parkhill', 720, 4000),
        ('parkhill', 'cherrycreek', 720, 4000),
        -- Park Hill <-> Highlands: ~20 min, 8 km (across downtown - TIGHT!)
        ('parkhill', 'highlands', 1200, 8000),
        ('highlands', 'parkhill', 1200, 8000),
        -- Highlands <-> Aurora: ~25 min, 20 km (across town)
        ('highlands', 'aurora', 1500, 20000),
        ('aurora', 'highlands', 1500, 20000),
        -- Aurora <-> Downtown: ~25 min, 18 km (VERY TIGHT for 30 min gap!)
        ('aurora', 'downtown', 1500, 18000),
        ('downtown', 'aurora', 1500, 18000),
        -- Downtown <-> Highlands: ~10 min, 3 km
        ('downtown', 'highlands', 600, 3000),
        ('highlands', 'downtown', 600, 3000),
        -- Downtown <-> Park Hill: ~15 min, 6 km
        ('downtown', 'parkhill', 900, 6000),
        ('parkhill', 'downtown', 900, 6000),
        -- Cherry Creek <-> Highlands: ~18 min, 7 km
        ('cherrycreek', 'highlands', 1080, 7000),
        ('highlands', 'cherrycreek', 1080, 7000),
        -- Cherry Creek <-> Aurora: ~20 min, 15 km
        ('cherrycreek', 'aurora', 1200, 15000),
        ('aurora', 'cherrycreek', 1200, 15000),
        -- Park Hill <-> Aurora: ~15 min, 10 km
        ('parkhill', 'aurora', 900, 10000),
        ('aurora', 'parkhill', 900, 10000)
) AS t(origin_name, dest_name, travel_seconds, travel_meters)
JOIN locs o ON o.loc_name = t.origin_name
JOIN locs d ON d.loc_name = t.dest_name
ON CONFLICT (origin_location_id, destination_location_id)
DO UPDATE SET travel_seconds = EXCLUDED.travel_seconds, distance_meters = EXCLUDED.distance_meters, calculated_at = NOW();

-- Verification queries
\echo '=== Customers Created ==='
SELECT email, first_name, last_name FROM users WHERE email LIKE '%@demo.com' AND role = 'customer' ORDER BY email;

\echo ''
\echo '=== Locations Created ==='
SELECT u.first_name, l.name, l.address, l.city, l.latitude, l.longitude
FROM locations l
JOIN users u ON l.user_id = u.id
WHERE u.email LIKE '%@demo.com' AND u.role = 'customer'
ORDER BY u.first_name;

\echo ''
\echo '=== Bookings for Tomorrow ==='
SELECT
    b.scheduled_start::time as start_time,
    b.scheduled_end::time as end_time,
    u.first_name || ' ' || u.last_name as customer,
    l.address,
    l.city,
    b.notes
FROM bookings b
JOIN users u ON b.customer_id = u.id
JOIN locations l ON b.location_id = l.id
WHERE b.walker_id = 'b376c762-b772-4fde-963e-5dcaedd52626'
AND DATE(b.scheduled_start) = CURRENT_DATE + 1
ORDER BY b.scheduled_start;

\echo ''
\echo '=== Travel Time Cache ==='
SELECT
    lo.address as from_location,
    ld.address as to_location,
    ttc.travel_seconds / 60 as travel_minutes,
    ttc.distance_meters / 1000.0 as distance_km
FROM travel_time_cache ttc
JOIN locations lo ON ttc.origin_location_id = lo.id
JOIN locations ld ON ttc.destination_location_id = ld.id
ORDER BY lo.address, ld.address
LIMIT 20;
