# PRD: Recurring Bookings Reliability & Zone Grouping

## Introduction

This PRD addresses reliability gaps identified in the recurring bookings feature and introduces zone-based group walks. Currently, recurring booking creation lacks transaction atomicity (partial failures possible), errors are silently swallowed, conflict detection makes N+1 queries, and there's no duplicate prevention on retry. Additionally, we're adding zone grouping to enable nearby customers to join group walks after dog meet-and-greets.

## Goals

- Ensure atomic creation of recurring series + all booking instances (all-or-nothing)
- Surface all booking conflicts and failures clearly to customers
- Reduce conflict detection from O(n) queries to O(1) batch query
- Prevent duplicate bookings on client retry via idempotency
- Add metrics and structured logging for operational visibility
- Enable zone-based grouping for group walks with compatibility verification

## User Stories

### US-001: Wrap recurring booking creation in database transaction
**Description:** As a system, I need atomic transactions so that partial failures don't leave orphaned series or missing bookings.

**Acceptance Criteria:**
- [ ] All recurring booking creation wrapped in single database transaction
- [ ] If any booking fails to create, entire transaction rolls back
- [ ] Series is only committed if all bookings succeed
- [ ] Existing availability check remains outside transaction (read-only)
- [ ] Typecheck passes
- [ ] Unit test verifies rollback on simulated failure

---

### US-002: Surface booking conflicts to customer with inline errors
**Description:** As a customer, I want to see exactly which dates have conflicts so I can make informed decisions about my recurring booking.

**Acceptance Criteria:**
- [ ] Conflicts displayed inline in booking form after submission attempt
- [ ] Each conflict shows: date, time, reason (e.g., "Walker unavailable", "Existing booking")
- [ ] Customer can see which dates WILL succeed vs which will be skipped
- [ ] Clear call-to-action: "Create X available bookings" or "Cancel"
- [ ] Typecheck passes
- [ ] Verify in browser

---

### US-003: Show toast notification for booking failures
**Description:** As a customer, I want immediate feedback when something goes wrong so I know my action didn't silently fail.

**Acceptance Criteria:**
- [ ] Toast appears on any booking error (network, server, validation)
- [ ] Toast includes specific error message, not generic "Something went wrong"
- [ ] Toast has "Retry" button for transient failures
- [ ] Toast auto-dismisses after 8 seconds but can be manually dismissed
- [ ] Multiple errors stack vertically
- [ ] Typecheck passes
- [ ] Verify in browser

---

### US-004: Batch conflict detection query
**Description:** As a system, I need to check all occurrence dates for conflicts in a single query to reduce database load.

**Acceptance Criteria:**
- [ ] Single query checks conflicts for all occurrence dates
- [ ] Query uses date range + walker_id to fetch potential conflicts
- [ ] In-memory filtering determines exact conflicts per occurrence
- [ ] Performance: <100ms for 52-week recurring booking check
- [ ] Typecheck passes
- [ ] Unit test compares results with current N-query approach

---

### US-005: Add client-generated idempotency key
**Description:** As a customer, I want my booking to be created exactly once even if I accidentally click submit twice or my connection drops and retries.

**Acceptance Criteria:**
- [ ] Frontend generates UUID for each booking submission
- [ ] UUID sent in `X-Idempotency-Key` header
- [ ] Backend stores idempotency key with series record
- [ ] Duplicate key returns original response, not error
- [ ] Keys expire after 24 hours
- [ ] Typecheck passes

---

### US-006: Add database constraint for booking uniqueness
**Description:** As a system, I need a database-level safeguard against duplicate bookings for the same customer, time, and service.

**Acceptance Criteria:**
- [ ] Add unique constraint on (customer_id, service_id, scheduled_start) where status != 'cancelled'
- [ ] Constraint is partial index excluding cancelled bookings
- [ ] Migration handles any existing duplicates gracefully
- [ ] API returns clear error message on constraint violation
- [ ] Typecheck passes

---

### US-007: Add structured logging for booking operations
**Description:** As an operator, I need structured logs to diagnose booking issues and understand system behavior.

**Acceptance Criteria:**
- [ ] Log booking creation with: customer_id, series_id, booking_count, conflicts_count
- [ ] Log each conflict with: date, reason, walker_id
- [ ] Log transaction outcomes: commit/rollback with duration_ms
- [ ] Use tracing spans for operation grouping
- [ ] Log level INFO for success, WARN for conflicts, ERROR for failures
- [ ] Typecheck passes

---

### US-008: Add booking metrics
**Description:** As an operator, I need metrics to monitor booking system health and identify trends.

**Acceptance Criteria:**
- [ ] Counter: `bookings_created_total` with labels (type: single/recurring, status: success/failure)
- [ ] Counter: `booking_conflicts_total` with labels (reason: overlap/block/travel_time)
- [ ] Histogram: `booking_creation_duration_seconds`
- [ ] Gauge: `active_recurring_series_count`
- [ ] Metrics exposed on `/metrics` endpoint
- [ ] Typecheck passes

---

### US-009: Integrate Google Maps API for travel time calculation
**Description:** As a system, I need accurate travel time estimates between appointments to prevent overbooking walkers.

**Acceptance Criteria:**
- [ ] Call Google Maps Distance Matrix API for travel time between locations
- [ ] Cache travel times for 24 hours (locations don't move)
- [ ] Fallback to straight-line distance estimate if API fails
- [ ] Add configurable buffer time (default: 15 minutes) on top of travel time
- [ ] Include traffic estimates for departure time
- [ ] Typecheck passes
- [ ] Integration test with mocked API

---

### US-010: Create zone model for geographic grouping
**Description:** As an admin, I need to define geographic zones so walkers can efficiently serve nearby customers.

**Acceptance Criteria:**
- [ ] New `zones` table with: id, name, organization_id, center_lat, center_lng, radius_meters
- [ ] Zones can overlap (customer may be in multiple zones)
- [ ] Admin can CRUD zones via API
- [ ] Location automatically assigned to zones based on coordinates
- [ ] Typecheck passes

---

### US-011: Enable group walk booking type
**Description:** As a customer, I want to book a group walk at a reduced rate if my dog is compatible with other dogs in my zone.

**Acceptance Criteria:**
- [ ] New service type: "group_walk" with max_dogs field
- [ ] Group walks only available within defined zones
- [ ] Price reduced (configurable percentage) for group walks
- [ ] Customer sees "Group walk available in your area" prompt
- [ ] Booking shows other dogs in group (names only, no owner info)
- [ ] Typecheck passes
- [ ] Verify in browser

---

### US-012: Dog compatibility tracking for group walks
**Description:** As a walker, I need to record dog meet-and-greet results so only compatible dogs are grouped together.

**Acceptance Criteria:**
- [ ] New `dog_compatibility` table: dog_a_id, dog_b_id, status (pending/compatible/incompatible), notes, verified_by, verified_at
- [ ] Walker can record compatibility after meet-and-greet
- [ ] Compatibility is bidirectional (A compatible with B = B compatible with A)
- [ ] Group walk only offers dogs marked as compatible
- [ ] Customer notified when meet-and-greet required before joining group
- [ ] Typecheck passes
- [ ] Verify in browser

---

### US-013: Group walk slot matching
**Description:** As a system, I need to match customers into group walk slots based on zone, compatibility, and timing.

**Acceptance Criteria:**
- [ ] When customer requests group walk, find existing slots in their zone within 30-min window
- [ ] Only show slots where their dog is compatible with all current dogs
- [ ] If no compatible slot exists, offer to start a new group
- [ ] Walker sees grouped bookings as single calendar block
- [ ] Maximum dogs per group enforced (from service config)
- [ ] Typecheck passes

---

## Functional Requirements

- FR-1: Recurring booking creation must be atomic - all bookings created or none
- FR-2: API must return detailed conflict information, not just success/failure
- FR-3: Frontend must display conflicts inline with actionable options
- FR-4: Toast notifications must appear for all error states with retry capability
- FR-5: Conflict detection must complete in single database round-trip
- FR-6: Idempotency key must prevent duplicate series creation on retry
- FR-7: Database constraint must prevent duplicate bookings at data layer
- FR-8: All booking operations must emit structured logs with correlation IDs
- FR-9: Prometheus metrics must be exposed for monitoring dashboards
- FR-10: Travel time between locations must include real traffic estimates
- FR-11: Zones must be defined with center point and radius
- FR-12: Group walks must enforce dog compatibility requirements
- FR-13: Meet-and-greet must be required before dogs can be grouped

## Non-Goals

- Real-time GPS tracking during walks (future feature)
- Automatic zone suggestion based on booking density
- AI-based dog compatibility prediction
- Payment processing changes (existing flow works)
- Walker route optimization across multiple group walks
- Push notifications (separate PRD)

## Technical Considerations

### Transaction Handling
Use `sqlx::Transaction` to wrap series + booking creation:
```rust
let mut tx = pool.begin().await?;
// Create series
// Create all bookings
tx.commit().await?;
```

### Batch Conflict Query
```sql
SELECT * FROM bookings
WHERE walker_id = $1
  AND scheduled_start >= $2
  AND scheduled_start <= $3
  AND status NOT IN ('cancelled')
```
Then filter in-memory against occurrence dates.

### Idempotency Storage
Add to `recurring_booking_series`:
```sql
idempotency_key UUID UNIQUE,
idempotency_expires_at TIMESTAMPTZ
```

### Google Maps Integration
- Use Distance Matrix API for batch travel time requests
- Store API key in environment variable
- Implement circuit breaker for API failures

### Zone Geometry
Use PostGIS for accurate geographic queries:
```sql
ST_DWithin(location.point, zone.center, zone.radius_meters)
```

## Design Considerations

### Error Display
- Inline errors appear below the date selector
- Conflicts shown in collapsible list with expand/collapse all
- Color coding: red for hard conflicts, yellow for warnings
- Toast appears top-right, stacks downward

### Group Walk UI
- Badge on service card: "Group walk - Save 20%"
- Compatibility status shown with dog avatars
- Meet-and-greet scheduling integrated into booking flow

## Success Metrics

- Zero partial booking creations (transaction integrity)
- 99% of errors surfaced to customer (vs silently swallowed)
- Conflict check latency <100ms for 52-week series
- Zero duplicate bookings from retry (idempotency working)
- 15% of bookings become group walks within 6 months (zone adoption)
- Meet-and-greet conversion rate >80%

## Open Questions

1. Should we allow customers to opt-out of group walks permanently?
2. What happens if a dog's compatibility status changes after booking?
3. Should zones be visible to customers or internal-only?
4. How do we handle group walk cancellation - notify other participants?
5. Should travel time cache be per-time-of-day (rush hour vs off-peak)?

---

## Implementation Sequence

### Phase 1: Reliability (P0 + P1)
1. US-001: Transaction wrapper
2. US-004: Batch conflict detection
3. US-005: Client idempotency key
4. US-006: Database uniqueness constraint
5. US-002: Inline error display
6. US-003: Toast notifications

### Phase 2: Observability
7. US-007: Structured logging
8. US-008: Metrics

### Phase 3: Travel Time & Zones
9. US-009: Google Maps integration
10. US-010: Zone model

### Phase 4: Group Walks
11. US-011: Group walk booking type
12. US-012: Dog compatibility tracking
13. US-013: Group walk slot matching
