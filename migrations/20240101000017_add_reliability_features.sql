-- Add idempotency support to recurring booking series
ALTER TABLE recurring_booking_series
ADD COLUMN idempotency_key UUID UNIQUE,
ADD COLUMN idempotency_expires_at TIMESTAMPTZ;

-- Create index for idempotency key lookups
CREATE INDEX idx_recurring_series_idempotency ON recurring_booking_series(idempotency_key) WHERE idempotency_key IS NOT NULL;

-- Add partial unique constraint to prevent duplicate bookings
-- Only applies to non-cancelled bookings for the same customer, service, and time
CREATE UNIQUE INDEX idx_booking_uniqueness ON bookings (customer_id, service_id, scheduled_start)
WHERE status NOT IN ('cancelled');

-- Add comment explaining the constraint
COMMENT ON INDEX idx_booking_uniqueness IS 'Prevents duplicate bookings for same customer, service, and start time (excludes cancelled)';
