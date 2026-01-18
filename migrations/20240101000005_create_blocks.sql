-- Create blocks table (for lunch, personal time, etc.)
CREATE TABLE blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    walker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason VARCHAR(255) NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    is_recurring BOOLEAN NOT NULL DEFAULT false,
    recurrence_rule TEXT, -- iCal RRULE format
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_block_times CHECK (end_time > start_time)
);

-- Indexes
CREATE INDEX idx_blocks_walker_id ON blocks(walker_id);
CREATE INDEX idx_blocks_time_range ON blocks(walker_id, start_time, end_time);
