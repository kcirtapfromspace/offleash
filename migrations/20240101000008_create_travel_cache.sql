-- Create travel time cache table
-- Used to cache travel time estimates between locations
CREATE TABLE travel_cache (
    id BIGSERIAL PRIMARY KEY,
    origin_location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    destination_location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    duration_minutes INTEGER NOT NULL CHECK (duration_minutes >= 0),
    distance_meters INTEGER NOT NULL CHECK (distance_meters >= 0),
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    UNIQUE (origin_location_id, destination_location_id)
);

-- Indexes
CREATE INDEX idx_travel_cache_lookup ON travel_cache(origin_location_id, destination_location_id);
CREATE INDEX idx_travel_cache_expiry ON travel_cache(expires_at);
