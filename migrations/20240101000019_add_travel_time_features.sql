-- Travel time cache for storing calculated travel times between locations
CREATE TABLE travel_time_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    origin_location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    destination_location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    travel_seconds INTEGER NOT NULL,
    distance_meters INTEGER NOT NULL,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(origin_location_id, destination_location_id)
);

-- Index for quick lookups
CREATE INDEX idx_travel_time_cache_lookup
ON travel_time_cache(origin_location_id, destination_location_id);

-- Index for cache invalidation (find stale entries)
CREATE INDEX idx_travel_time_cache_age
ON travel_time_cache(calculated_at);

-- Walker live location for real-time availability calculations
CREATE TABLE walker_locations (
    walker_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    latitude DECIMAL(10, 8) NOT NULL,
    longitude DECIMAL(11, 8) NOT NULL,
    accuracy_meters DECIMAL(8, 2),
    heading DECIMAL(5, 2), -- Direction of travel in degrees
    speed_mps DECIMAL(6, 2), -- Speed in meters per second
    is_on_duty BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for finding on-duty walkers
CREATE INDEX idx_walker_locations_on_duty
ON walker_locations(is_on_duty)
WHERE is_on_duty = true;

-- Add travel buffer configuration to organizations
ALTER TABLE organizations
ADD COLUMN travel_buffer_minutes INTEGER NOT NULL DEFAULT 15;

-- Add comments for documentation
COMMENT ON TABLE travel_time_cache IS 'Cached travel times between locations to reduce Google Maps API calls';
COMMENT ON TABLE walker_locations IS 'Live location tracking for walkers when on duty';
COMMENT ON COLUMN organizations.travel_buffer_minutes IS 'Default buffer time added around appointments for travel';
COMMENT ON COLUMN walker_locations.is_on_duty IS 'Whether walker is currently accepting real-time bookings';
