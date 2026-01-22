-- Calendar scheduling schema for walker availability and external calendar sync

-- Calendar event types
CREATE TYPE calendar_event_type AS ENUM ('booking', 'block', 'personal', 'synced');

-- Calendar provider types
CREATE TYPE calendar_provider AS ENUM ('google', 'apple', 'caldav');

-- Sync direction options
CREATE TYPE sync_direction AS ENUM ('push', 'pull', 'bidirectional');

-- Sync status for tracking
CREATE TYPE sync_status AS ENUM ('pending', 'synced', 'failed', 'conflict');

-- Calendar events table (blocks, synced events - bookings reference this)
CREATE TABLE calendar_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255),
    description TEXT,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    all_day BOOLEAN NOT NULL DEFAULT false,
    event_type calendar_event_type NOT NULL,
    -- For synced events: reference to external calendar
    calendar_connection_id UUID,
    external_event_id VARCHAR(255),
    -- Sync tracking
    sync_status sync_status NOT NULL DEFAULT 'pending',
    last_synced_at TIMESTAMPTZ,
    -- For recurring blocks
    recurrence_rule TEXT, -- iCal RRULE format
    recurrence_parent_id UUID REFERENCES calendar_events(id) ON DELETE CASCADE,
    -- Metadata
    color VARCHAR(7), -- Hex color for display
    is_blocking BOOLEAN NOT NULL DEFAULT true, -- Whether this blocks booking availability
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_time_range CHECK (end_time > start_time)
);

-- Calendar connections (external calendar integrations)
CREATE TABLE calendar_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider calendar_provider NOT NULL,
    -- OAuth tokens (encrypted at application level)
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    -- CalDAV credentials (encrypted at application level)
    server_url TEXT,
    username TEXT,
    password_encrypted TEXT,
    -- Calendar selection
    calendar_id VARCHAR(255) NOT NULL, -- External calendar ID
    calendar_name VARCHAR(255), -- Display name
    calendar_color VARCHAR(7), -- Hex color from provider
    -- Sync settings
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    sync_direction sync_direction NOT NULL DEFAULT 'bidirectional',
    push_bookings BOOLEAN NOT NULL DEFAULT true,
    push_blocks BOOLEAN NOT NULL DEFAULT false,
    -- Sync state
    last_sync_at TIMESTAMPTZ,
    sync_token TEXT, -- For incremental sync (Google uses syncToken, CalDAV uses ctag)
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_calendar UNIQUE (user_id, provider, calendar_id)
);

-- Calendar sync log for debugging and audit
CREATE TABLE calendar_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    connection_id UUID NOT NULL REFERENCES calendar_connections(id) ON DELETE CASCADE,
    direction sync_direction NOT NULL,
    status sync_status NOT NULL,
    events_created INTEGER NOT NULL DEFAULT 0,
    events_updated INTEGER NOT NULL DEFAULT 0,
    events_deleted INTEGER NOT NULL DEFAULT 0,
    conflicts_detected INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX idx_calendar_events_user_date ON calendar_events(user_id, start_time, end_time);
CREATE INDEX idx_calendar_events_org_date ON calendar_events(organization_id, start_time, end_time);
CREATE INDEX idx_calendar_events_type ON calendar_events(event_type);
CREATE INDEX idx_calendar_events_external ON calendar_events(calendar_connection_id, external_event_id)
    WHERE external_event_id IS NOT NULL;
CREATE INDEX idx_calendar_events_recurrence ON calendar_events(recurrence_parent_id)
    WHERE recurrence_parent_id IS NOT NULL;

CREATE INDEX idx_calendar_connections_user ON calendar_connections(user_id);
CREATE INDEX idx_calendar_connections_sync ON calendar_connections(sync_enabled, last_sync_at)
    WHERE sync_enabled = true;

CREATE INDEX idx_calendar_sync_logs_connection ON calendar_sync_logs(connection_id, created_at DESC);

-- Add calendar_event_id to bookings for linking
ALTER TABLE bookings ADD COLUMN calendar_event_id UUID REFERENCES calendar_events(id) ON DELETE SET NULL;
CREATE INDEX idx_bookings_calendar_event ON bookings(calendar_event_id) WHERE calendar_event_id IS NOT NULL;

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_calendar_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER calendar_events_updated_at
    BEFORE UPDATE ON calendar_events
    FOR EACH ROW EXECUTE FUNCTION update_calendar_updated_at();

CREATE TRIGGER calendar_connections_updated_at
    BEFORE UPDATE ON calendar_connections
    FOR EACH ROW EXECUTE FUNCTION update_calendar_updated_at();
