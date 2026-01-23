# PRD: Walker Calendar Scheduling

## Introduction

Implement a full-featured calendar system that allows walkers to manage their availability and bookings with a familiar Google/Apple Calendar UX. The system provides two-way sync with external calendars (Google Calendar, Apple Calendar/iCloud, and CalDAV-compatible apps), enabling walkers to use their preferred calendar while keeping everything in sync. Customers can view available slots and their bookings through both the admin dashboard and iOS app.

## Goals

- Provide familiar calendar UX (day/week/month/agenda views) in admin dashboard and iOS app
- Enable two-way sync with Google Calendar and Apple Calendar (iCloud)
- Support CalDAV standard for broader calendar app compatibility
- Allow walkers to manually mark unavailable times
- Smart detection of "busy" vs personal events from synced calendars
- Give customers visibility into available slots and their booking history
- Reduce scheduling conflicts and double-bookings

## User Stories

### Phase 1: Core Calendar UI

#### US-001: Add calendar database schema
**Description:** As a developer, I need to store calendar events and sync metadata so the system can track availability and external calendar connections.

**Acceptance Criteria:**
- [ ] Create `calendar_events` table with: id, organization_id, user_id, title, description, start_time, end_time, all_day, event_type (booking|block|personal|synced), external_calendar_id, external_event_id, sync_status, created_at, updated_at
- [ ] Create `calendar_connections` table with: id, user_id, provider (google|apple|caldav), access_token (encrypted), refresh_token (encrypted), calendar_id, sync_enabled, last_sync_at, sync_direction (push|pull|bidirectional)
- [ ] Create `calendar_sync_log` table for debugging: id, connection_id, direction, status, events_synced, error_message, created_at
- [ ] Add indexes for efficient date-range queries
- [ ] Migration runs successfully
- [ ] Typecheck passes

#### US-002: Calendar week view component (Admin Dashboard)
**Description:** As a walker, I want to see my schedule in a week view so I can understand my upcoming appointments at a glance.

**Acceptance Criteria:**
- [ ] Week view displays 7 days with hourly time slots (6am-10pm default)
- [ ] Bookings displayed as colored blocks with customer name and service type
- [ ] Blocked times shown in gray with diagonal stripes
- [ ] Synced external events shown with calendar icon indicator
- [ ] Current time indicator (red line) on today's column
- [ ] Click on empty slot opens "Create Block" modal
- [ ] Click on event opens event detail popover
- [ ] Navigation arrows to move between weeks
- [ ] "Today" button to jump to current week
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-003: Calendar day view component (Admin Dashboard)
**Description:** As a walker, I want a detailed day view so I can see my full schedule for a specific day.

**Acceptance Criteria:**
- [ ] Day view shows single day with 30-minute time slots
- [ ] Events display with full details (customer, service, location, notes)
- [ ] Drag-and-drop to reschedule events (bookings require confirmation)
- [ ] Drag edges to resize event duration
- [ ] Double-click empty slot to create block
- [ ] Date picker to jump to specific date
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-004: Calendar month view component (Admin Dashboard)
**Description:** As a walker, I want a month overview so I can see busy vs free days and plan ahead.

**Acceptance Criteria:**
- [ ] Month grid shows all days with event count indicators
- [ ] Days with bookings show colored dots (count badge if >3)
- [ ] Days with blocks show gray indicator
- [ ] Click on day navigates to day view
- [ ] Hover on day shows tooltip with event summary
- [ ] Previous/next month navigation
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-005: Calendar agenda view component (Admin Dashboard)
**Description:** As a walker, I want a list view of upcoming events so I can quickly scan my schedule.

**Acceptance Criteria:**
- [ ] Chronological list of events grouped by day
- [ ] Shows next 14 days by default, with "Load more" option
- [ ] Each event shows: time, customer name, service, location
- [ ] Filter by event type: All | Bookings | Blocks | Synced
- [ ] Empty state for days with no events
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-006: Create availability block (Admin Dashboard)
**Description:** As a walker, I want to manually block times when I'm unavailable so customers can't book those slots.

**Acceptance Criteria:**
- [ ] "Block Time" button in calendar toolbar
- [ ] Modal with: title (optional), start date/time, end date/time, recurring options
- [ ] Recurring options: None | Daily | Weekly | Weekdays | Custom
- [ ] Custom recurrence: select specific days, end date or occurrence count
- [ ] Preview shows affected dates before saving
- [ ] Block appears immediately on calendar after save
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-007: Edit/delete calendar events (Admin Dashboard)
**Description:** As a walker, I want to modify or remove blocks so I can adjust my availability.

**Acceptance Criteria:**
- [ ] Click event opens detail popover with Edit/Delete buttons
- [ ] Edit opens modal pre-filled with event data
- [ ] For recurring events: "Edit this event" vs "Edit all future events" choice
- [ ] Delete confirmation dialog
- [ ] For recurring: "Delete this event" vs "Delete all future events"
- [ ] Cannot edit/delete synced events (shows "Edit in [Calendar Name]" link)
- [ ] Typecheck passes
- [ ] Verify in browser

### Phase 2: External Calendar Integration

#### US-008: Google Calendar OAuth connection
**Description:** As a walker, I want to connect my Google Calendar so my personal events sync with my work schedule.

**Acceptance Criteria:**
- [ ] "Connect Google Calendar" button in settings
- [ ] OAuth 2.0 flow with calendar.readonly and calendar.events scopes
- [ ] After auth, show list of user's calendars to select which to sync
- [ ] Store encrypted tokens in database
- [ ] Show connected status with "Disconnect" option
- [ ] Handle token refresh automatically
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-009: Apple Calendar (iCloud) connection
**Description:** As a walker, I want to connect my Apple Calendar so my iPhone calendar syncs with my work schedule.

**Acceptance Criteria:**
- [ ] "Connect Apple Calendar" button in settings
- [ ] App-specific password flow (Apple doesn't support OAuth for CalDAV)
- [ ] Instructions for generating app-specific password
- [ ] CalDAV connection to iCloud calendar server
- [ ] Select which calendars to sync
- [ ] Store credentials securely (encrypted)
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-010: CalDAV generic connection
**Description:** As a walker, I want to connect any CalDAV calendar so I can use my preferred calendar app.

**Acceptance Criteria:**
- [ ] "Connect Other Calendar (CalDAV)" option
- [ ] Form fields: Server URL, username, password
- [ ] Auto-discover calendars from CalDAV server
- [ ] Test connection before saving
- [ ] Select calendars to sync
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-011: Pull sync from external calendars
**Description:** As a walker, I want external calendar events to appear in my work calendar so I can see all commitments in one place.

**Acceptance Criteria:**
- [ ] Background job polls connected calendars every 15 minutes
- [ ] New external events create local "synced" events
- [ ] Updated external events update local copies
- [ ] Deleted external events remove local copies
- [ ] Smart detection: events marked "busy" or with certain keywords block availability
- [ ] Events marked "free" or "tentative" don't block availability
- [ ] Sync status indicator in calendar header
- [ ] Manual "Sync Now" button
- [ ] Typecheck passes

#### US-012: Push sync to external calendars
**Description:** As a walker, I want my work bookings to appear in my personal calendar so I see everything in one place.

**Acceptance Criteria:**
- [ ] When booking is created, push event to connected calendar(s)
- [ ] Event includes: customer name, service type, location address, booking notes
- [ ] When booking is updated, update external event
- [ ] When booking is cancelled, remove from external calendar
- [ ] Blocks optionally sync (user preference)
- [ ] Include link back to admin dashboard in event description
- [ ] Typecheck passes

#### US-013: Conflict detection with smart merge
**Description:** As a walker, I want the system to intelligently detect scheduling conflicts so I don't get double-booked.

**Acceptance Criteria:**
- [ ] When external event syncs, check for booking conflicts
- [ ] "Busy" events automatically block those time slots
- [ ] "Free/Tentative" events show warning but don't block
- [ ] Personal events (detected by keywords: lunch, doctor, personal, etc.) block by default
- [ ] Work events from other jobs show as conflicts
- [ ] Conflict indicator on affected bookings
- [ ] Daily email summary of detected conflicts (optional)
- [ ] Typecheck passes

### Phase 3: iOS App Calendar

#### US-014: Walker calendar view (iOS)
**Description:** As a walker using the iOS app, I want to see my schedule so I can manage my day on the go.

**Acceptance Criteria:**
- [ ] Tab bar item "Schedule" with calendar icon
- [ ] Default view: Agenda (list) for mobile-friendly UX
- [ ] Toggle to Day view for detailed single-day view
- [ ] Toggle to Week view (horizontal scroll)
- [ ] Pull-to-refresh syncs with server
- [ ] Tap event to see details
- [ ] Swipe actions: Call customer, Get directions, Cancel
- [ ] Typecheck passes
- [ ] Verify in iOS Simulator

#### US-015: Create block from iOS app
**Description:** As a walker, I want to block time from my phone so I can update availability on the go.

**Acceptance Criteria:**
- [ ] "+" button to create new block
- [ ] Quick options: "Block next hour", "Block rest of day", "Custom"
- [ ] Custom: date/time pickers for start and end
- [ ] Optional title field
- [ ] Recurring options (same as web)
- [ ] Confirmation before saving
- [ ] Typecheck passes
- [ ] Verify in iOS Simulator

#### US-016: Customer booking calendar (iOS)
**Description:** As a customer using the iOS app, I want to see my upcoming bookings in a calendar view.

**Acceptance Criteria:**
- [ ] Calendar view showing days with bookings highlighted
- [ ] Tap day to see booking details
- [ ] Upcoming bookings list below calendar
- [ ] Each booking shows: date, time, walker name, service, status
- [ ] Tap booking to see full details with cancel option
- [ ] Empty state when no upcoming bookings
- [ ] Typecheck passes
- [ ] Verify in iOS Simulator

#### US-017: Customer available slots view (iOS)
**Description:** As a customer, I want to see when walkers are available so I can book at convenient times.

**Acceptance Criteria:**
- [ ] During booking flow, show calendar with available slots
- [ ] Green = available, Gray = unavailable
- [ ] Tap day to see available time slots
- [ ] Time slots show walker availability (if multiple walkers)
- [ ] Selected slot highlights and enables "Continue" button
- [ ] Integrates with existing booking flow
- [ ] Typecheck passes
- [ ] Verify in iOS Simulator

### Phase 4: Admin Dashboard Customer View

#### US-018: Customer booking calendar (Admin Dashboard)
**Description:** As a customer using the admin dashboard, I want to see my bookings in a calendar format.

**Acceptance Criteria:**
- [ ] Calendar view in customer dashboard
- [ ] Month view with booking indicators
- [ ] Click day to see bookings for that day
- [ ] Booking cards show: time, walker, service, status
- [ ] Filter by status: Upcoming | Past | Cancelled
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-019: Available slots picker (Admin Dashboard)
**Description:** As a customer, I want to pick from available time slots when booking so I can find a convenient time.

**Acceptance Criteria:**
- [ ] Step in booking flow: "Select Date & Time"
- [ ] Calendar shows available days (green) vs fully booked (gray)
- [ ] Selecting day shows available time slots
- [ ] Slots grouped by walker if multiple available
- [ ] Show walker rating/reviews next to their slots
- [ ] Selected slot persists to next step
- [ ] Typecheck passes
- [ ] Verify in browser

## Functional Requirements

- FR-1: Calendar events stored with organization_id for multi-tenant isolation
- FR-2: All calendar views respect user timezone (stored in user profile)
- FR-3: External calendar tokens encrypted at rest using AES-256
- FR-4: Calendar sync runs as background job, not blocking user requests
- FR-5: Sync failures logged and retried with exponential backoff
- FR-6: Two-way sync uses etag/sync tokens to detect changes efficiently
- FR-7: Calendar UI components shared between walker and customer views
- FR-8: Mobile calendar optimized for touch (larger tap targets, swipe gestures)
- FR-9: Drag-and-drop on desktop uses native HTML5 drag API
- FR-10: Real-time updates via WebSocket when calendar changes (optional enhancement)

## Non-Goals (Out of Scope)

- Video call integration (Zoom/Meet links in events)
- SMS/push notifications for calendar reminders (separate feature)
- Shared team calendars (walkers see only their own)
- Calendar embedding/widget for external websites
- Outlook/Exchange calendar integration (CalDAV covers most cases)
- Recurring bookings auto-creation (handled by existing recurring booking feature)
- Payment integration in calendar (handled by booking flow)

## Design Considerations

- Use existing calendar libraries where possible:
  - Admin Dashboard: Consider FullCalendar.js or similar
  - iOS: Use native EventKit for calendar UI patterns
- Color coding consistent across platforms:
  - Blue: Confirmed bookings
  - Yellow: Pending bookings
  - Gray: Blocks/unavailable
  - Purple: Synced external events
- Calendar toolbar: View switcher, date navigation, today button, sync status
- Mobile-first for iOS: Agenda view default, day/week as secondary
- Accessibility: Keyboard navigation, screen reader support, high contrast mode

## Technical Considerations

- Google Calendar API: Use google-calendar-api crate or REST API directly
- Apple Calendar: CalDAV protocol via caldav crate
- Token refresh: Background job to refresh OAuth tokens before expiry
- Sync conflict resolution: Server timestamp wins, log conflicts for review
- Performance: Index on (organization_id, user_id, start_time) for fast queries
- Caching: Cache external calendar data locally, refresh on sync
- Rate limiting: Respect Google/Apple API rate limits

## Success Metrics

- Walkers can view and modify their schedule in under 3 clicks
- External calendar sync completes within 30 seconds
- Zero double-bookings due to sync lag (conflicts detected and blocked)
- 80% of walkers connect at least one external calendar within first week
- Calendar page load time under 500ms

## Open Questions

- Should we support importing existing external calendar events as bookings?
- How do we handle timezone changes when walker travels?
- Should customers see specific walker availability or just "available slots"?
- Do we need offline support for iOS calendar view?
- Should recurring blocks support exceptions (e.g., "every Tuesday except Jan 30")?
