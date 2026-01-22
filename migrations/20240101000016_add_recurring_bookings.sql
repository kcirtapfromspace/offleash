-- Recurrence frequency enum
CREATE TYPE recurrence_frequency AS ENUM ('weekly', 'bi_weekly', 'monthly');

-- Recurring series table (stores the "recipe" for recurring bookings)
CREATE TABLE recurring_booking_series (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    customer_id UUID NOT NULL REFERENCES users(id),
    walker_id UUID NOT NULL REFERENCES users(id),
    service_id UUID NOT NULL REFERENCES services(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    frequency recurrence_frequency NOT NULL,
    day_of_week INTEGER NOT NULL CHECK (day_of_week >= 0 AND day_of_week <= 6),
    time_of_day TIME NOT NULL,
    timezone VARCHAR(50) NOT NULL DEFAULT 'America/Denver',
    end_date DATE,
    total_occurrences INTEGER CHECK (total_occurrences > 0),
    is_active BOOLEAN NOT NULL DEFAULT true,
    price_cents_per_booking BIGINT NOT NULL CHECK (price_cents_per_booking >= 0),
    default_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_end_condition CHECK (
        (end_date IS NOT NULL AND total_occurrences IS NULL) OR
        (end_date IS NULL AND total_occurrences IS NOT NULL)
    )
);

-- Link bookings to series
ALTER TABLE bookings
ADD COLUMN recurring_series_id UUID REFERENCES recurring_booking_series(id),
ADD COLUMN occurrence_number INTEGER;

-- Indexes for recurring series
CREATE INDEX idx_recurring_series_org ON recurring_booking_series(organization_id);
CREATE INDEX idx_recurring_series_customer ON recurring_booking_series(customer_id);
CREATE INDEX idx_recurring_series_walker ON recurring_booking_series(walker_id);
CREATE INDEX idx_recurring_series_active ON recurring_booking_series(is_active) WHERE is_active = true;

-- Index for bookings linked to series
CREATE INDEX idx_bookings_recurring_series ON bookings(recurring_series_id) WHERE recurring_series_id IS NOT NULL;
