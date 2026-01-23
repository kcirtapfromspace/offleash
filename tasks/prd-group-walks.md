# PRD: Group Walks with Meet & Greet Enrollment

## Introduction

Enable walkers to offer group walk services where multiple dogs walk together. Dogs must pass a meet & greet assessment before being eligible for group walks. Walkers can post recurring or one-time group walk slots, and approved customers can enroll their dogs. This creates a new revenue stream, builds community, and provides socialization opportunities for dogs.

## Goals

- Allow walkers to create group walk offerings with configurable capacity and pricing
- Gate group walk enrollment behind a meet & greet assessment
- Support recurring scheduled groups and ad-hoc group formation
- Enable customers to discover and join available group walks
- Track dog compatibility and group dynamics

## User Stories

### US-301: Create Group Walk Service Type
**Description:** As a business owner, I want to create group walk services with flexible pricing and capacity settings.

**Acceptance Criteria:**
- [ ] New service type: "Group Walk"
- [ ] Configurable max dogs per group (2-8)
- [ ] Pricing model options: per-dog flat rate, tiered pricing, or percentage discount
- [ ] Minimum dogs required to run group (e.g., 2)
- [ ] Duration setting (30, 45, 60 min options)
- [ ] Service can require meet & greet approval
- [ ] Typecheck passes
- [ ] Verify in browser

### US-302: Walker Posts Group Walk Slots
**Description:** As a walker, I want to post available group walk times so customers can join.

**Acceptance Criteria:**
- [ ] Create one-time group walk slot (date, time, location, capacity)
- [ ] Create recurring group walk (e.g., Mon/Wed/Fri 10am)
- [ ] Set meeting point location (park, trail, etc.)
- [ ] Add description/notes about the walk
- [ ] See current enrollment count
- [ ] Cancel slot (notifies enrolled customers)
- [ ] Typecheck passes
- [ ] Verify in browser

### US-303: Meet & Greet Assessment
**Description:** As a walker, I want to assess dogs before approving them for group walks.

**Acceptance Criteria:**
- [ ] Schedule meet & greet appointment (new booking type)
- [ ] Assessment form with criteria:
  - Leash behavior
  - Dog reactivity (friendly, selective, reactive)
  - Recall reliability
  - Size/energy level compatibility notes
- [ ] Pass/Fail/Conditional outcome
- [ ] Notes field for special instructions
- [ ] Approval grants "Group Walk Eligible" status to dog
- [ ] Typecheck passes
- [ ] Verify in browser

### US-304: Dog Group Eligibility Status
**Description:** As a customer, I want to see if my dog is eligible for group walks and what's needed if not.

**Acceptance Criteria:**
- [ ] Dog profile shows group walk eligibility status
- [ ] Statuses: Not Assessed, Pending, Approved, Not Approved
- [ ] If not assessed, prompt to book meet & greet
- [ ] Approval expiration option (e.g., valid for 1 year)
- [ ] Walker can revoke eligibility with reason
- [ ] Typecheck passes
- [ ] Verify in browser

### US-305: Customer Enrolls in Group Walk
**Description:** As a customer with an approved dog, I want to join an available group walk.

**Acceptance Criteria:**
- [ ] Browse available group walks (list/calendar view)
- [ ] See: date, time, location, spots remaining, walker, price
- [ ] Filter by location, date range
- [ ] Enroll button (only for eligible dogs)
- [ ] Select which dog(s) to enroll (if multiple approved)
- [ ] Confirmation with meeting point details
- [ ] Add to calendar option
- [ ] Typecheck passes
- [ ] Verify in browser

### US-306: Group Walk Roster Management
**Description:** As a walker, I want to see who's enrolled in my group walks and manage the roster.

**Acceptance Criteria:**
- [ ] View enrolled dogs with owner contact info
- [ ] See dog photos, names, and any notes
- [ ] Remove dog from roster (with notification to owner)
- [ ] Mark attendance (showed/no-show)
- [ ] Add walk notes per dog
- [ ] Typecheck passes
- [ ] Verify in browser

### US-307: Recurring Group Walk Schedule
**Description:** As a walker, I want to set up recurring group walks that customers can enroll in ongoing.

**Acceptance Criteria:**
- [ ] Create recurring schedule (weekly on selected days)
- [ ] Customers enroll for ongoing participation
- [ ] Handle holidays/exceptions (skip dates)
- [ ] Enrollment carries forward week to week
- [ ] Customer can unenroll from future walks
- [ ] Typecheck passes
- [ ] Verify in browser

### US-308: Group Walk Pricing Configuration
**Description:** As a business owner, I want flexible pricing for group walks.

**Acceptance Criteria:**
- [ ] Base price per dog per walk
- [ ] Optional: tiered pricing (1st dog $30, 2nd dog same owner $20)
- [ ] Optional: package pricing (10-walk punch card)
- [ ] Minimum booking fee if group doesn't fill
- [ ] Cancellation policy settings
- [ ] Typecheck passes
- [ ] Verify in browser

### US-309: Group Walk Completion
**Description:** As a walker, I want to complete a group walk and record what happened.

**Acceptance Criteria:**
- [ ] Start walk (check-in all attending dogs)
- [ ] GPS route tracking during walk
- [ ] End walk with summary
- [ ] Individual notes per dog
- [ ] Group photo upload
- [ ] Report card sent to each owner
- [ ] Typecheck passes

### US-310: Customer Views Group Walk History
**Description:** As a customer, I want to see my dog's group walk history and upcoming enrollments.

**Acceptance Criteria:**
- [ ] List of upcoming enrolled group walks
- [ ] History of past group walks with notes
- [ ] See which dogs were in the group
- [ ] View photos from walks
- [ ] Typecheck passes
- [ ] Verify in browser

## Functional Requirements

- FR-1: New service category "Group Walk" with capacity settings
- FR-2: Meet & greet is a special booking type that grants eligibility
- FR-3: Dog eligibility stored on pet record with expiration date
- FR-4: Group walk slots are separate from regular booking slots
- FR-5: Enrollment creates booking records linked to group walk
- FR-6: Minimum enrollment threshold to confirm group (configurable)
- FR-7: Auto-cancel notification if minimum not met 24hrs before
- FR-8: Recurring groups use same enrollment model as recurring bookings
- FR-9: Walker can set "compatible dogs only" restrictions
- FR-10: Price calculated at enrollment time, charged normally

## Non-Goals

- No automated dog matching/compatibility algorithms (walker decides)
- No customer-to-customer messaging (only via walker)
- No real-time GPS sharing with customers during walk (MVP)
- No dynamic pricing based on enrollment count
- No waitlist management (first-come-first-served)

## Technical Considerations

### Database Schema
```sql
-- Group walk eligibility on pets
ALTER TABLE pets ADD COLUMN group_walk_eligible BOOLEAN DEFAULT false;
ALTER TABLE pets ADD COLUMN group_walk_assessed_at TIMESTAMPTZ;
ALTER TABLE pets ADD COLUMN group_walk_assessed_by UUID REFERENCES users(id);
ALTER TABLE pets ADD COLUMN group_walk_notes TEXT;
ALTER TABLE pets ADD COLUMN group_walk_expires_at TIMESTAMPTZ;

-- Meet & greet assessment records
CREATE TABLE meet_greet_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pet_id UUID NOT NULL REFERENCES pets(id),
    walker_id UUID NOT NULL REFERENCES users(id),
    booking_id UUID REFERENCES bookings(id),

    -- Assessment criteria
    leash_behavior VARCHAR(20), -- excellent, good, needs_work, poor
    dog_reactivity VARCHAR(20), -- friendly, selective, reactive
    recall_reliability VARCHAR(20), -- excellent, good, needs_work, poor
    energy_level VARCHAR(20), -- low, medium, high
    size_category VARCHAR(20), -- small, medium, large, xlarge

    outcome VARCHAR(20) NOT NULL, -- approved, not_approved, conditional
    notes TEXT,
    conditions TEXT, -- if conditional, what conditions

    assessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Group walk slots (what walkers post)
CREATE TABLE group_walk_slots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    walker_id UUID NOT NULL REFERENCES users(id),
    service_id UUID NOT NULL REFERENCES services(id),
    location_id UUID NOT NULL REFERENCES locations(id),

    scheduled_date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,

    max_dogs INTEGER NOT NULL DEFAULT 6,
    min_dogs INTEGER NOT NULL DEFAULT 2,
    current_enrollment INTEGER NOT NULL DEFAULT 0,

    price_cents BIGINT NOT NULL,

    status VARCHAR(20) NOT NULL DEFAULT 'open', -- open, full, confirmed, cancelled, completed

    description TEXT,
    meeting_point_notes TEXT,

    -- For recurring slots
    recurring_series_id UUID,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Group walk enrollments (customers joining)
CREATE TABLE group_walk_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_walk_slot_id UUID NOT NULL REFERENCES group_walk_slots(id),
    customer_id UUID NOT NULL REFERENCES users(id),
    pet_id UUID NOT NULL REFERENCES pets(id),
    booking_id UUID REFERENCES bookings(id), -- links to payment

    status VARCHAR(20) NOT NULL DEFAULT 'enrolled', -- enrolled, cancelled, attended, no_show

    enrolled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cancelled_at TIMESTAMPTZ,

    walker_notes TEXT, -- per-dog notes from the walk

    UNIQUE(group_walk_slot_id, pet_id)
);

-- Recurring group walk series
CREATE TABLE recurring_group_walks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    walker_id UUID NOT NULL REFERENCES users(id),
    service_id UUID NOT NULL REFERENCES services(id),
    location_id UUID NOT NULL REFERENCES locations(id),

    days_of_week INTEGER[] NOT NULL, -- [1,3,5] for Mon/Wed/Fri
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,

    max_dogs INTEGER NOT NULL DEFAULT 6,
    min_dogs INTEGER NOT NULL DEFAULT 2,
    price_cents BIGINT NOT NULL,

    description TEXT,
    meeting_point_notes TEXT,

    is_active BOOLEAN NOT NULL DEFAULT true,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for finding available slots
CREATE INDEX idx_group_walk_slots_available
ON group_walk_slots(scheduled_date, status)
WHERE status IN ('open', 'confirmed');
```

### API Endpoints
```
# Group Walk Slots
GET    /group-walks                     # List available slots
POST   /group-walks                     # Create slot (walker)
GET    /group-walks/:id                 # Slot details with roster
PUT    /group-walks/:id                 # Update slot
DELETE /group-walks/:id                 # Cancel slot

# Enrollment
POST   /group-walks/:id/enroll          # Enroll dog
DELETE /group-walks/:id/enrollments/:id # Unenroll
PUT    /group-walks/:id/enrollments/:id # Update (attendance, notes)

# Meet & Greet
POST   /meet-greets                     # Create assessment
GET    /meet-greets/:petId              # Get pet's assessment history
PUT    /pets/:id/group-eligibility      # Update eligibility directly

# Recurring
POST   /group-walks/recurring           # Create recurring series
GET    /group-walks/recurring           # List walker's recurring
PUT    /group-walks/recurring/:id       # Update series
DELETE /group-walks/recurring/:id       # End series
```

### Service Configuration
```json
{
  "name": "Group Walk",
  "type": "group_walk",
  "duration_minutes": 60,
  "max_dogs": 6,
  "min_dogs": 2,
  "price_per_dog_cents": 2500,
  "requires_meet_greet": true,
  "meet_greet_validity_days": 365
}
```

## Design Considerations

### Customer Booking Flow
1. Customer browses "Group Walks" section
2. Sees available slots with spots remaining
3. If dog not eligible: "Book a Meet & Greet first" CTA
4. If eligible: "Join This Walk" → select dog → confirm → pay
5. Confirmation shows meeting point, walker contact, what to bring

### Walker Dashboard
- "My Group Walks" section showing upcoming slots
- Quick view of enrollment count per slot
- Roster management with dog photos
- "Create Group Walk" prominent action

### Admin Dashboard
- Group walk analytics (fill rates, popular times)
- Meet & greet conversion tracking
- Revenue from group walks vs individual

## Success Metrics

- 30% of active customers try group walks within 6 months
- Average group fill rate > 70%
- Meet & greet to enrollment conversion > 80%
- Group walk revenue grows to 20% of total revenue
- Customer retention higher for group walk participants

## Open Questions

1. Should we show other dogs in the group before enrolling (privacy)?
2. How to handle one dog having issues mid-walk (behavioral)?
3. Should customers be able to request specific group mates?
4. Insurance implications for group walks?
5. What if walker is sick - substitute walker for group?

## Implementation Phases

### Phase 1: Foundation
- Database schema
- Group walk service type
- Basic slot creation (one-time)
- Enrollment flow

### Phase 2: Meet & Greet
- Assessment booking type
- Assessment form
- Eligibility tracking
- Customer visibility

### Phase 3: Customer Experience
- Group walk discovery/browse
- Enrollment flow
- Upcoming walks view
- Walk history

### Phase 4: Recurring & Polish
- Recurring group walks
- Ongoing enrollment
- Analytics
- Walk completion flow with notes/photos
