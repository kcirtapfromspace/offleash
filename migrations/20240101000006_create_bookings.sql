-- Create booking status enum
CREATE TYPE booking_status AS ENUM (
    'pending', 'confirmed', 'in_progress', 'completed', 'cancelled', 'no_show'
);

-- Create bookings table
CREATE TABLE bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES users(id),
    walker_id UUID NOT NULL REFERENCES users(id),
    service_id UUID NOT NULL REFERENCES services(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    status booking_status NOT NULL DEFAULT 'pending',
    scheduled_start TIMESTAMPTZ NOT NULL,
    scheduled_end TIMESTAMPTZ NOT NULL,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_scheduled_times CHECK (scheduled_end > scheduled_start),
    CONSTRAINT valid_actual_times CHECK (
        actual_end IS NULL OR actual_start IS NULL OR actual_end > actual_start
    )
);

-- Indexes
CREATE INDEX idx_bookings_customer_id ON bookings(customer_id);
CREATE INDEX idx_bookings_walker_id ON bookings(walker_id);
CREATE INDEX idx_bookings_status ON bookings(status);
CREATE INDEX idx_bookings_scheduled_start ON bookings(scheduled_start);

-- Composite index for availability queries
CREATE INDEX idx_bookings_walker_schedule ON bookings(walker_id, scheduled_start, scheduled_end)
    WHERE status NOT IN ('cancelled', 'completed');
