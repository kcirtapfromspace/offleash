# PRD: Dynamic Timeslot Booking with Travel Time Intelligence

## Introduction

Enable smart booking that accounts for real-world travel constraints. When customers book a walk, available timeslots should reflect actual walker availability including travel time from their current location (via iOS app) or from their previous appointment. This prevents overbooking, reduces walker stress, and improves on-time reliability.

## Goals

- Show customers only realistically available timeslots based on travel time
- Use live walker location from iOS app for same-day bookings
- Calculate travel time between consecutive appointments
- Add configurable buffer time for travel
- Warn customers when travel time is tight
- Improve on-time arrival rate to 95%+

## User Stories

### US-201: Calculate Travel Time Between Appointments
**Description:** As a system, I need to calculate travel time between a walker's appointments so I can determine realistic availability.

**Acceptance Criteria:**
- [ ] Given two locations, calculate driving time via Google Maps API
- [ ] Cache travel times for common route pairs (15-minute TTL)
- [ ] Handle API failures gracefully (use distance-based estimate)
- [ ] Account for service duration + travel time when checking availability
- [ ] Typecheck passes

### US-202: Live Walker Location for Same-Day Bookings
**Description:** As a customer booking a same-day walk, I want timeslots based on where my walker actually is right now.

**Acceptance Criteria:**
- [ ] iOS app reports walker location when on duty
- [ ] Backend receives location updates (every 5 min or on significant move)
- [ ] Same-day booking slots calculated from current location
- [ ] Location data expires after 30 minutes of no updates
- [ ] Falls back to last known appointment location if no live data
- [ ] Typecheck passes

### US-203: Smart Timeslot Filtering
**Description:** As a customer, I should only see timeslots where the walker can realistically arrive on time.

**Acceptance Criteria:**
- [ ] Timeslots hidden if walker cannot arrive within buffer window
- [ ] Consider: previous appointment end time + travel + buffer
- [ ] Consider: walker's working hours start/end
- [ ] Consider: existing blocked time on calendar
- [ ] Show "No available times" message if none qualify
- [ ] Typecheck passes
- [ ] Verify in browser

### US-204: Travel Buffer Configuration
**Description:** As a business owner, I want to configure how much buffer time to add around appointments for travel.

**Acceptance Criteria:**
- [ ] Setting for default travel buffer (e.g., 15 minutes)
- [ ] Option to set buffer per service type (longer for distant locations)
- [ ] Buffer applied before AND after appointment slots
- [ ] Admin can override for specific walkers
- [ ] Typecheck passes
- [ ] Verify in browser

### US-205: Travel Time Warnings
**Description:** As a customer, I want to see if my booking requires significant travel so I can choose appropriately.

**Acceptance Criteria:**
- [ ] Timeslots show travel time indicator (e.g., "15 min travel")
- [ ] Warning badge on slots with >20 min travel
- [ ] Tooltip explains "Walker traveling from previous appointment"
- [ ] Tight slots (travel + buffer barely fits) show caution indicator
- [ ] Typecheck passes
- [ ] Verify in browser

### US-206: Walker Schedule Optimization View
**Description:** As a walker, I want to see my daily route optimized with travel times between appointments.

**Acceptance Criteria:**
- [ ] Daily view shows appointments with travel time between each
- [ ] Total travel time for day displayed
- [ ] Map view option showing route
- [ ] Warning if schedule is too tight
- [ ] Typecheck passes
- [ ] Verify in browser (admin dashboard)

### US-207: iOS Location Reporting
**Description:** As a walker using the iOS app, my location should be shared when I'm on duty for accurate booking availability.

**Acceptance Criteria:**
- [ ] Location tracking when walker marks "On Duty"
- [ ] Battery-efficient updates (significant change or 5-min interval)
- [ ] Clear indicator that location is being shared
- [ ] Option to pause location sharing
- [ ] Location stops when "Off Duty"
- [ ] Privacy notice and consent on first enable

## Functional Requirements

- FR-1: Integrate Google Maps Distance Matrix API for travel time calculation
- FR-2: Cache travel times with 15-minute TTL to reduce API calls
- FR-3: Store walker live location in Redis with 30-minute expiry
- FR-4: Default travel buffer: 15 minutes (configurable per org)
- FR-5: Calculate availability as: `slot_start >= prev_end + travel_time + buffer`
- FR-6: For same-day bookings within 2 hours, use live location
- FR-7: For future bookings, use last scheduled appointment location
- FR-8: iOS app uses significant-change location monitoring (battery efficient)
- FR-9: Fallback to straight-line distance × 2 min/mile if API fails
- FR-10: Log all travel time calculations for optimization analysis

## Non-Goals

- No multi-stop route optimization (just pairwise travel times)
- No traffic prediction for future dates (use average times)
- No public transit travel times (driving only)
- No customer location tracking
- No automatic schedule rearrangement

## Technical Considerations

### New Database Tables
```sql
-- Travel time cache
CREATE TABLE travel_time_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    origin_location_id UUID REFERENCES locations(id),
    destination_location_id UUID REFERENCES locations(id),
    travel_seconds INTEGER NOT NULL,
    distance_meters INTEGER NOT NULL,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(origin_location_id, destination_location_id)
);

-- Walker live location
CREATE TABLE walker_locations (
    walker_id UUID PRIMARY KEY REFERENCES users(id),
    latitude DECIMAL(10, 8) NOT NULL,
    longitude DECIMAL(11, 8) NOT NULL,
    accuracy_meters DECIMAL(8, 2),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_on_duty BOOLEAN NOT NULL DEFAULT false
);

-- Organization travel settings
ALTER TABLE organizations ADD COLUMN travel_buffer_minutes INTEGER DEFAULT 15;
```

### API Endpoints
```
POST /walkers/:id/location          # iOS reports location
GET  /walkers/:id/location          # Get current location (admin)
GET  /availability/slots            # Enhanced with travel time
    ?walker_id=
    &location_id=
    &date=
    &service_id=
POST /travel-time/calculate         # Manual calculation
GET  /walkers/:id/daily-route       # Route optimization view
```

### iOS App Changes
- Add location permission request with usage description
- Background location updates when on duty
- Location indicator in status bar
- On/Off duty toggle persists location state

### Availability Algorithm
```
function getAvailableSlots(walker, date, serviceLocation, serviceDuration):
    slots = []
    workingHours = getWorkingHours(walker, date)
    existingBookings = getBookings(walker, date)
    blockedTimes = getBlockedTimes(walker, date)

    for each potential slot in workingHours:
        # Find previous commitment
        prevEnd = getPreviousCommitmentEnd(slot.start, existingBookings, blockedTimes)
        prevLocation = getPreviousLocation(slot.start, existingBookings)

        # Calculate travel time
        if isToday(date) and withinTwoHours(slot.start):
            originLocation = getLiveLocation(walker) or prevLocation
        else:
            originLocation = prevLocation

        travelTime = getTravelTime(originLocation, serviceLocation)
        buffer = getBufferMinutes(walker.org)

        # Check if slot is feasible
        earliestArrival = prevEnd + travelTime + buffer
        if slot.start >= earliestArrival:
            slot.travelMinutes = travelTime
            slot.isTight = (slot.start - earliestArrival) < 10 minutes
            slots.append(slot)

    return slots
```

## Design Considerations

### Customer Booking UI
- Timeslots show travel indicator: `10:00 AM (12 min travel)`
- Tight slots have yellow warning: `10:30 AM ⚠️ (tight schedule)`
- Unavailable times are hidden (not grayed out)
- "Why aren't more times available?" expandable explanation

### Walker Daily View (Admin/iOS)
- Timeline with travel blocks shown between appointments
- Color coding: Green (plenty of time), Yellow (tight), Red (late risk)
- Total stats: "5 appointments, 47 min total travel"

## Success Metrics

- On-time arrival rate improves from current baseline to 95%+
- Zero double-bookings due to travel time conflicts
- Average travel time per day decreases (smarter clustering)
- Customer complaints about late arrivals decrease 50%

## Open Questions

1. Should we factor in parking time for urban areas?
2. How to handle traffic for same-day bookings (live traffic data)?
3. Should walkers see "suggested route order" for efficiency?
4. What happens if walker location is stale (>30 min old)?

## Implementation Phases

### Phase 1: Travel Time Calculation
- Google Maps API integration
- Travel time caching
- Basic availability filtering

### Phase 2: Live Location
- iOS location reporting
- Redis storage for live locations
- Same-day booking enhancement

### Phase 3: UI Enhancements
- Travel time indicators in booking flow
- Walker daily route view
- Warning badges

### Phase 4: Optimization
- Travel analytics dashboard
- Route clustering suggestions
- Buffer time recommendations based on data
