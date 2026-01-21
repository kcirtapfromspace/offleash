# PRD: iOS App Authentication, Reliability & Observability Improvements

## Introduction

The OFFLEASH iOS app has critical gaps in authentication state management, performance optimization, and observability that directly impact customer experience. This initiative addresses findings from an Amazon-style executive code review, fixing a broken authentication recovery flow, adding caching for performance, implementing server-side filtering, and establishing observability infrastructure to measure customer impact.

**Problem Statement:** Customers can get stranded in a "phantom authenticated" state where the app thinks they're logged in but the backend has invalidated their session. There's no caching (causing unnecessary wait times), no observability (we can't measure customer experience), and security hardening is missing.

## Goals

- **Eliminate authentication dead-ends:** Customers should never be stranded due to expired/invalid tokens
- **Reduce time-to-interactive:** Implement persistent caching to show content faster on repeat visits
- **Enable data-driven decisions:** Instrument key customer journey metrics
- **Improve security posture:** Add certificate pinning and input validation
- **Reduce payload sizes:** Move filtering server-side to improve performance at scale

## User Stories

### Phase 1: Critical Auth Fixes (P0)

#### US-001: Session Recovery on 401 Errors
**Description:** As a customer, when my session expires I should be automatically redirected to login so I can continue using the app.

**Acceptance Criteria:**
- [ ] When APIClient receives a 401 response, publish a notification/event
- [ ] OFFLEASHApp observes auth state changes and navigates to login screen
- [ ] Customer sees a brief toast/alert explaining "Session expired, please log in again"
- [ ] Any in-progress work (e.g., booking flow) state is cleared gracefully
- [ ] Typecheck passes
- [ ] Verify in simulator: force 401 response, confirm redirect to login

#### US-002: Token Validation on App Launch
**Description:** As a customer, when I open the app with a stored token, it should verify the token is still valid before showing authenticated content.

**Acceptance Criteria:**
- [ ] On app launch, if token exists, call `GET /auth/validate` endpoint
- [ ] Show loading state during validation (not a blank screen)
- [ ] If valid: proceed to ContentView
- [ ] If invalid/expired: clear token and show LoginView
- [ ] If network error: show cached content with offline indicator (graceful degradation)
- [ ] Typecheck passes
- [ ] Verify in simulator: test with valid token, expired token, and airplane mode

#### US-003: Backend - Add Token Validation Endpoint
**Description:** As the iOS app, I need a lightweight endpoint to validate tokens without fetching full user data.

**Acceptance Criteria:**
- [ ] `GET /auth/validate` endpoint exists
- [ ] Returns 200 with `{ "valid": true, "expiresAt": "ISO8601" }` for valid tokens
- [ ] Returns 401 for invalid/expired tokens
- [ ] Response time < 100ms P99
- [ ] Endpoint is documented in API spec
- [ ] Unit tests pass

### Phase 2: Performance & Caching (P1)

#### US-004: Implement Persistent Cache Layer
**Description:** As a developer, I need a reusable caching layer that persists data to disk with TTL support.

**Acceptance Criteria:**
- [ ] Create `CacheManager` actor with generic support for Codable types
- [ ] Support configurable TTL per cache key
- [ ] Persist to app's Caches directory using FileManager
- [ ] Automatic cache invalidation on TTL expiry
- [ ] Cache size limit with LRU eviction (max 50MB)
- [ ] Typecheck passes
- [ ] Unit tests for cache hit, miss, expiry, and eviction

#### US-005: Cache Services List
**Description:** As a customer, I want to see the services list instantly on repeat visits instead of waiting for network.

**Acceptance Criteria:**
- [ ] ServicesView checks cache before making network request
- [ ] If cache hit and not expired (TTL: 5 minutes): show cached data immediately
- [ ] Background refresh after showing cached data (stale-while-revalidate pattern)
- [ ] If cache miss: show loading state, fetch from network, cache response
- [ ] Pull-to-refresh bypasses cache and forces network fetch
- [ ] Typecheck passes
- [ ] Verify in simulator: load services, kill app, relaunch, confirm instant display

#### US-006: Cache Locations List
**Description:** As a customer, I want to see my saved locations instantly instead of waiting for network.

**Acceptance Criteria:**
- [ ] LocationSelectionView checks cache before making network request
- [ ] Cache TTL: 10 minutes (locations change less frequently)
- [ ] Same stale-while-revalidate pattern as services
- [ ] Cache invalidated when customer adds/edits/deletes a location
- [ ] Typecheck passes
- [ ] Verify in simulator: load locations, kill app, relaunch, confirm instant display

#### US-007: Server-Side Active Filtering for Services
**Description:** As a developer, I want to filter inactive services on the server to reduce payload size.

**Acceptance Criteria:**
- [ ] Backend: Add `?active=true` query parameter to `GET /services`
- [ ] When `active=true`, only return services where `isActive=true`
- [ ] iOS: Update APIClient call to include `?active=true`
- [ ] iOS: Remove client-side `.filter { $0.isActive }` from ServicesView
- [ ] Measure payload size reduction (target: document before/after)
- [ ] Typecheck passes
- [ ] API tests pass

### Phase 3: Observability (P2)

#### US-008: Integrate Analytics SDK
**Description:** As a product manager, I need analytics infrastructure to measure customer behavior.

**Acceptance Criteria:**
- [ ] Integrate analytics SDK (Firebase Analytics, Amplitude, or Mixpanel)
- [ ] Configure in OFFLEASHApp on launch
- [ ] Respect user privacy settings (no tracking if opted out)
- [ ] Typecheck passes
- [ ] Verify events appear in analytics dashboard

#### US-009: Track Screen View Events
**Description:** As a product manager, I want to know which screens customers visit and how long they spend.

**Acceptance Criteria:**
- [ ] Track screen view event when each major view appears: Login, Register, Services, LocationSelection, Booking
- [ ] Include timestamp and session ID
- [ ] Track time-on-screen when navigating away
- [ ] Typecheck passes
- [ ] Verify in analytics dashboard

#### US-010: Track Time-to-Interactive Metrics
**Description:** As an engineer, I want to measure how long customers wait for content to load.

**Acceptance Criteria:**
- [ ] Track time from view appearance to content displayed (loading complete)
- [ ] Track separately: cache hit vs network fetch
- [ ] Log P50, P90, P99 to analytics
- [ ] Create dashboard showing TTI trends over time
- [ ] Typecheck passes

#### US-011: Track Error Events
**Description:** As an engineer, I want to know when and where customers encounter errors.

**Acceptance Criteria:**
- [ ] Track all APIError occurrences with: error type, endpoint, HTTP status, timestamp
- [ ] Track retry attempts and outcomes
- [ ] Redact any PII from error payloads
- [ ] Create alert for error rate spike (>5% of requests in 5-minute window)
- [ ] Typecheck passes

#### US-012: Track Funnel Progression
**Description:** As a product manager, I want to measure conversion through the booking funnel.

**Acceptance Criteria:**
- [ ] Define funnel stages: Login/Register → Services → Location → Booking Start → Booking Complete
- [ ] Track progression and drop-off at each stage
- [ ] Include customer segment (new vs returning)
- [ ] Create funnel visualization dashboard
- [ ] Typecheck passes

#### US-013: Integrate Crash Reporting
**Description:** As an engineer, I want to be notified of app crashes with stack traces.

**Acceptance Criteria:**
- [ ] Integrate crash reporting SDK (Firebase Crashlytics or Sentry)
- [ ] Configure in OFFLEASHApp on launch
- [ ] Include device info, OS version, app version in crash reports
- [ ] Set up PagerDuty/Slack alerts for crash rate spikes
- [ ] Typecheck passes
- [ ] Verify test crash appears in dashboard

### Phase 4: Security Hardening (P3)

#### US-014: Implement Certificate Pinning
**Description:** As a security engineer, I want to prevent MITM attacks on API communication.

**Acceptance Criteria:**
- [ ] Pin API server's TLS certificate in APIClient
- [ ] Use backup pins for certificate rotation
- [ ] Fail closed (reject connection) on pin mismatch
- [ ] Document certificate rotation procedure
- [ ] Typecheck passes
- [ ] Test with Charles proxy: verify connection is rejected

#### US-015: Strengthen Email Validation
**Description:** As a customer, I want clear feedback if my email format is invalid before submitting.

**Acceptance Criteria:**
- [ ] Use RFC 5322 compliant email regex validation
- [ ] Show inline validation error below email field
- [ ] Validation runs on field blur and on submit
- [ ] Error message: "Please enter a valid email address"
- [ ] Match backend validation rules exactly
- [ ] Typecheck passes
- [ ] Verify in simulator: test edge cases (no @, multiple @, missing domain)

#### US-016: Add Password Strength Requirements
**Description:** As a customer, I want to understand password requirements during registration.

**Acceptance Criteria:**
- [ ] Display password requirements below field: min 8 chars, 1 uppercase, 1 number
- [ ] Real-time validation checkmarks as requirements are met
- [ ] Prevent submission until all requirements met
- [ ] Match backend validation rules exactly
- [ ] Typecheck passes
- [ ] Verify in simulator

## Functional Requirements

### Authentication
- FR-1: APIClient must publish auth state changes via NotificationCenter or Combine
- FR-2: OFFLEASHApp must observe auth state and navigate accordingly
- FR-3: Token validation must occur on every cold app launch
- FR-4: Backend must provide `/auth/validate` endpoint with <100ms P99 latency

### Caching
- FR-5: CacheManager must persist to disk in Caches directory
- FR-6: CacheManager must support configurable TTL per key
- FR-7: CacheManager must enforce 50MB size limit with LRU eviction
- FR-8: ServicesView must implement stale-while-revalidate pattern
- FR-9: LocationSelectionView must invalidate cache on data mutation

### Server-Side Filtering
- FR-10: Backend must support `?active=true` query parameter on `/services`
- FR-11: iOS must request only active services

### Observability
- FR-12: All major screens must track view events
- FR-13: All API errors must be logged with context
- FR-14: Time-to-interactive must be measured for list views
- FR-15: Crash reports must include device and app version context

### Security
- FR-16: APIClient must implement certificate pinning
- FR-17: Email validation must match backend rules
- FR-18: Password requirements must be displayed and enforced client-side

## Non-Goals

- Token refresh/rotation mechanism (separate initiative)
- Offline-first architecture (caching is for performance, not offline support)
- Biometric authentication (Face ID/Touch ID)
- Rate limiting on client side
- End-to-end encryption of API payloads
- GDPR data export/deletion features

## Technical Considerations

### iOS
- Use Swift Concurrency (async/await) throughout
- CacheManager should be an actor for thread safety
- Use Combine or NotificationCenter for auth state propagation
- Analytics SDK should be abstracted behind protocol for testability

### Backend
- `/auth/validate` should only decode JWT, no database calls
- `?active=true` filter should use database index on `isActive` column
- Consider rate limiting on validation endpoint to prevent abuse

### Infrastructure
- Analytics: Firebase Analytics recommended (free tier sufficient for MVP)
- Crash Reporting: Firebase Crashlytics (integrates with Analytics)
- Alerting: Configure in Firebase Console or integrate with PagerDuty

### Dependencies
- Firebase iOS SDK (~10.x)
- No other new dependencies required

## Design Considerations

- Loading states should use existing ProgressView patterns
- Session expired toast should match app's visual language
- Password requirement checklist should use green checkmarks (SF Symbol: checkmark.circle.fill)
- Offline indicator should be subtle banner, not blocking modal

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Auth error recovery rate | Unknown (no tracking) | 100% auto-recovery |
| Services list TTI (cached) | N/A | <100ms |
| Services list TTI (network) | Unknown | <500ms P90 |
| 401 error rate | Unknown | <0.1% of sessions |
| Crash-free session rate | Unknown | >99.5% |
| Funnel: Services → Booking Start | Unknown | Establish baseline, then +10% |

## Open Questions

1. Which analytics provider is preferred? (Firebase recommended for simplicity)
2. What's the token TTL on the backend? (Needed to set appropriate cache TTLs)
3. Should we implement token refresh, or is re-authentication acceptable?
4. Who should receive crash/error alerts? (Engineering on-call rotation?)
5. Are there existing backend API docs, or should we generate them?

## Milestones

| Phase | Stories | Target |
|-------|---------|--------|
| Phase 1: Critical Auth Fixes | US-001, US-002, US-003 | Week 2 |
| Phase 2: Caching & Performance | US-004, US-005, US-006, US-007 | Week 4 |
| Phase 3: Observability | US-008 - US-013 | Week 8 |
| Phase 4: Security Hardening | US-014, US-015, US-016 | Week 10 |
