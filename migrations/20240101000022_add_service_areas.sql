-- Service Areas / Geofencing
-- Stores polygon boundaries for walker service areas

-- Service areas table
CREATE TABLE IF NOT EXISTS service_areas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    walker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Area details
    name VARCHAR(255) NOT NULL,
    color VARCHAR(7) DEFAULT '#3B82F6', -- Hex color code

    -- Polygon stored as array of lat/lng points (JSON array)
    -- Format: [{"lat": 39.7392, "lng": -104.9903}, ...]
    polygon JSONB NOT NULL,

    -- Bounding box for quick filtering (calculated from polygon)
    min_latitude DOUBLE PRECISION,
    max_latitude DOUBLE PRECISION,
    min_longitude DOUBLE PRECISION,
    max_longitude DOUBLE PRECISION,

    -- Area status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Priority (for overlapping areas, lower = higher priority)
    priority INTEGER DEFAULT 0,

    -- Pricing adjustment for this area (percentage, e.g., 10 = 10% extra)
    price_adjustment_percent INTEGER DEFAULT 0,

    -- Notes
    notes TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_service_areas_walker ON service_areas(walker_id);
CREATE INDEX idx_service_areas_org ON service_areas(organization_id);
CREATE INDEX idx_service_areas_active ON service_areas(is_active) WHERE is_active = TRUE;

-- Spatial index using bounding box for quick location filtering
CREATE INDEX idx_service_areas_bounds ON service_areas(min_latitude, max_latitude, min_longitude, max_longitude);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_service_area_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER service_areas_updated_at
    BEFORE UPDATE ON service_areas
    FOR EACH ROW
    EXECUTE FUNCTION update_service_area_updated_at();

-- Function to calculate bounding box from polygon
CREATE OR REPLACE FUNCTION calculate_service_area_bounds()
RETURNS TRIGGER AS $$
DECLARE
    point JSONB;
    lat DOUBLE PRECISION;
    lng DOUBLE PRECISION;
BEGIN
    -- Initialize with extreme values
    NEW.min_latitude := 90;
    NEW.max_latitude := -90;
    NEW.min_longitude := 180;
    NEW.max_longitude := -180;

    -- Iterate through polygon points
    FOR point IN SELECT * FROM jsonb_array_elements(NEW.polygon)
    LOOP
        lat := (point->>'lat')::DOUBLE PRECISION;
        lng := (point->>'lng')::DOUBLE PRECISION;

        IF lat < NEW.min_latitude THEN NEW.min_latitude := lat; END IF;
        IF lat > NEW.max_latitude THEN NEW.max_latitude := lat; END IF;
        IF lng < NEW.min_longitude THEN NEW.min_longitude := lng; END IF;
        IF lng > NEW.max_longitude THEN NEW.max_longitude := lng; END IF;
    END LOOP;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER service_areas_calculate_bounds
    BEFORE INSERT OR UPDATE OF polygon ON service_areas
    FOR EACH ROW
    EXECUTE FUNCTION calculate_service_area_bounds();

-- Helper function to check if a point is within a service area's bounding box
-- (Quick filter before more expensive polygon check)
CREATE OR REPLACE FUNCTION point_in_service_area_bounds(
    p_latitude DOUBLE PRECISION,
    p_longitude DOUBLE PRECISION,
    p_area_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    area_record service_areas%ROWTYPE;
BEGIN
    SELECT * INTO area_record FROM service_areas WHERE id = p_area_id;

    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;

    RETURN p_latitude >= area_record.min_latitude
       AND p_latitude <= area_record.max_latitude
       AND p_longitude >= area_record.min_longitude
       AND p_longitude <= area_record.max_longitude;
END;
$$ LANGUAGE plpgsql;

-- Function to find walkers who service a given location
CREATE OR REPLACE FUNCTION find_walkers_for_location(
    p_org_id UUID,
    p_latitude DOUBLE PRECISION,
    p_longitude DOUBLE PRECISION
) RETURNS TABLE(walker_id UUID, area_id UUID, area_name VARCHAR, priority INTEGER) AS $$
BEGIN
    RETURN QUERY
    SELECT sa.walker_id, sa.id, sa.name, sa.priority
    FROM service_areas sa
    WHERE sa.organization_id = p_org_id
      AND sa.is_active = TRUE
      AND p_latitude >= sa.min_latitude
      AND p_latitude <= sa.max_latitude
      AND p_longitude >= sa.min_longitude
      AND p_longitude <= sa.max_longitude
    ORDER BY sa.priority ASC;
END;
$$ LANGUAGE plpgsql;
