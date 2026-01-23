# PRD: External Calendar Sync

## Introduction

Enable walkers to connect their personal calendars (Google, Apple, Outlook) to automatically sync availability and blocked time. This allows walkers to manage their schedule from their preferred calendar app while keeping OFFLEASH updated with their true availability.

## Goals

- Allow walkers to connect Google, Apple (iCloud), and Outlook calendars via OAuth
- Two-way sync: pull external events as blocked time, push OFFLEASH bookings to external calendar
- Real-time or near-real-time sync via webhooks/push notifications
- Handle conflicts gracefully (external calendar is source of truth for personal events)
- Support multiple calendar connections per walker

## User Stories

### US-101: Connect Google Calendar
**Description:** As a walker, I want to connect my Google Calendar so my personal events automatically block my availability.

**Acceptance Criteria:**
- [ ] "Connect Google Calendar" button in settings/calendar page
- [ ] OAuth consent flow redirects to Google, returns to app
- [ ] Access token and refresh token stored securely
- [ ] Connection status shown (connected/disconnected)
- [ ] Can disconnect calendar at any time
- [ ] Typecheck passes
- [ ] Verify in browser

### US-102: Pull External Events as Blocked Time
**Description:** As a walker, I want my Google Calendar events to appear on my OFFLEASH calendar as blocked time.

**Acceptance Criteria:**
- [ ] Initial sync pulls events from connected calendar (next 3 months)
- [ ] External events display with "Synced" badge and different color
- [ ] Events marked as "busy" block booking availability
- [ ] Events marked as "free" do not block availability
- [ ] All-day events handled correctly
- [ ] Typecheck passes
- [ ] Verify in browser

### US-103: Push Bookings to External Calendar
**Description:** As a walker, I want my OFFLEASH bookings to appear on my Google Calendar automatically.

**Acceptance Criteria:**
- [ ] New bookings create events in connected calendar
- [ ] Event includes customer name, service, location, notes
- [ ] Booking cancellations remove/update external event
- [ ] Booking time changes update external event
- [ ] Typecheck passes

### US-104: Real-time Sync via Webhooks
**Description:** As a walker, I want changes in my Google Calendar to sync to OFFLEASH within minutes.

**Acceptance Criteria:**
- [ ] Register webhook with Google Calendar API
- [ ] Webhook endpoint receives push notifications
- [ ] Changes processed and calendar updated
- [ ] Fallback polling if webhook fails
- [ ] Typecheck passes

### US-105: Connect Apple Calendar (iCloud/CalDAV)
**Description:** As a walker, I want to connect my Apple Calendar for the same sync functionality.

**Acceptance Criteria:**
- [ ] "Connect Apple Calendar" option in settings
- [ ] App-specific password or OAuth flow for iCloud
- [ ] CalDAV protocol implementation for sync
- [ ] Same sync behavior as Google Calendar
- [ ] Typecheck passes
- [ ] Verify in browser

### US-106: Connect Outlook Calendar
**Description:** As a walker, I want to connect my Outlook/Microsoft 365 calendar.

**Acceptance Criteria:**
- [ ] "Connect Outlook" button using Microsoft Graph API
- [ ] OAuth flow with Microsoft identity platform
- [ ] Same sync behavior as Google Calendar
- [ ] Typecheck passes
- [ ] Verify in browser

### US-107: Manage Calendar Connections
**Description:** As a walker, I want to view and manage all my connected calendars.

**Acceptance Criteria:**
- [ ] List of connected calendars with status
- [ ] Last sync time displayed
- [ ] Manual "Sync Now" button
- [ ] Disconnect option with confirmation
- [ ] Sync error messages displayed clearly
- [ ] Typecheck passes
- [ ] Verify in browser

### US-108: Conflict Resolution
**Description:** As a walker, I want conflicts between external events and OFFLEASH bookings handled gracefully.

**Acceptance Criteria:**
- [ ] If external event conflicts with existing booking, show warning
- [ ] External events cannot delete confirmed bookings
- [ ] Admin notified of scheduling conflicts
- [ ] Conflict history/log available
- [ ] Typecheck passes

## Functional Requirements

- FR-1: Support OAuth 2.0 flow for Google and Microsoft calendars
- FR-2: Support CalDAV protocol for Apple Calendar/iCloud
- FR-3: Store OAuth tokens encrypted in `calendar_connections` table
- FR-4: Refresh tokens automatically before expiration
- FR-5: Initial sync fetches 3 months of future events
- FR-6: Ongoing sync via webhooks (Google/Outlook) or polling (CalDAV)
- FR-7: Sync frequency: real-time for webhooks, 15-minute polling fallback
- FR-8: Push OFFLEASH bookings to external calendar with structured event data
- FR-9: Handle token revocation gracefully (prompt reconnection)
- FR-10: Rate limit API calls per provider guidelines
- FR-11: Log all sync operations in `calendar_sync_log` table

## Non-Goals

- No support for generic ICS file import (only live calendar connections)
- No support for shared/team calendars (personal calendars only)
- No calendar selection UI (sync primary calendar only, for MVP)
- No historical sync (only future events)
- No support for recurring event modifications in external calendars (treat as individual events)

## Technical Considerations

### Database (Already Exists)
```sql
-- calendar_connections table stores OAuth tokens
-- calendar_sync_log tracks sync operations
-- calendar_events stores synced events with external_event_id
```

### New Backend Components Needed
```
crates/integrations/src/
├── google_calendar/
│   ├── mod.rs
│   ├── client.rs      # Google Calendar API client
│   ├── oauth.rs       # OAuth flow handlers
│   └── webhook.rs     # Push notification handler
├── apple_calendar/
│   ├── mod.rs
│   └── caldav.rs      # CalDAV client
├── outlook_calendar/
│   ├── mod.rs
│   ├── client.rs      # Microsoft Graph client
│   └── oauth.rs       # Microsoft OAuth flow
└── sync/
    ├── mod.rs
    ├── engine.rs      # Sync orchestration
    └── conflict.rs    # Conflict resolution
```

### API Endpoints Needed
```
GET  /calendar/connections           # List connections
POST /calendar/connections/google    # Start Google OAuth
GET  /calendar/connections/google/callback  # OAuth callback
POST /calendar/connections/apple     # Connect Apple (CalDAV credentials)
POST /calendar/connections/outlook   # Start Microsoft OAuth
GET  /calendar/connections/outlook/callback
DELETE /calendar/connections/:id     # Disconnect
POST /calendar/connections/:id/sync  # Manual sync trigger
POST /webhooks/google-calendar       # Google push notifications
POST /webhooks/outlook-calendar      # Microsoft notifications
```

### Environment Variables Needed
```
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
GOOGLE_REDIRECT_URI=

MICROSOFT_CLIENT_ID=
MICROSOFT_CLIENT_SECRET=
MICROSOFT_REDIRECT_URI=

# Apple uses app-specific passwords, no OAuth
```

### Security Considerations
- Store tokens encrypted at rest
- Use short-lived access tokens, refresh as needed
- Validate webhook signatures (Google, Microsoft)
- Scope OAuth permissions to minimum required (calendar read/write only)
- Never log full tokens

## Design Considerations

### Admin Dashboard UI
- Settings page with "Calendar Connections" section
- Provider icons (Google, Apple, Outlook) with connect buttons
- Connection status cards showing:
  - Provider name and icon
  - Connected account email
  - Last sync time
  - Sync status (synced, syncing, error)
  - Disconnect button

### Calendar View Updates
- Synced events show with cloud icon badge
- Different color/opacity for external vs local events
- Tooltip shows "Synced from Google Calendar"
- Cannot edit synced events directly (edit in source)

## Success Metrics

- 80% of active walkers connect at least one external calendar
- Sync latency < 5 minutes for webhook-enabled providers
- < 1% sync failure rate
- Zero double-bookings due to sync delays

## Open Questions

1. Should we support selecting specific calendars (work vs personal)?
2. How to handle all-day events that span multiple days?
3. Should synced events be editable in OFFLEASH (with sync back)?
4. What's the retry strategy for failed syncs?
5. How to handle timezone differences between calendars?

## Implementation Phases

### Phase 1: Google Calendar (Priority)
- OAuth flow
- Pull sync (external → OFFLEASH)
- Push sync (bookings → Google)
- Webhook real-time sync

### Phase 2: Apple Calendar
- CalDAV implementation
- Pull/push sync
- Polling-based sync (no webhooks)

### Phase 3: Outlook Calendar
- Microsoft Graph OAuth
- Pull/push sync
- Webhook support

### Phase 4: Polish
- Conflict resolution UI
- Sync history/logs
- Multi-calendar support
