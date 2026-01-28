# PRD: Comprehensive Test Suite for OFFLEASH

## Introduction

Create a complete end-to-end test suite for the OFFLEASH platform covering all functional requirements across the API, three web applications (customer-web, admin-dashboard, platform-admin), and iOS app. The test suite follows a **critical path first** approach, prioritizing authentication → booking → payment user journeys, with API tests implemented before UI tests.

The suite supports local development, CI/CD pipelines (GitHub Actions), and staging environment validation, with comprehensive multi-tenant testing including organization provisioning, user roles, and isolation verification.

---

## Goals

- Validate all critical user journeys end-to-end (auth, booking, payment)
- Ensure API contracts are stable between frontend and backend
- Verify multi-tenant isolation and context switching
- Catch regressions before deployment to staging/production
- Provide confidence for refactoring and new feature development
- Enable payment testing with mocks (default) and optional real provider integration
- Support testing across all platforms: Web (3 apps) + iOS

---

## User Stories

### Phase 1: Test Infrastructure Setup

#### US-001: Initialize Playwright Test Framework
**Description:** As a developer, I need Playwright configured for all web apps so I can write and run E2E tests.

**Acceptance Criteria:**
- [ ] Create `playwright.config.ts` in project root with multi-project setup
- [ ] Configure projects for: customer-web, admin-dashboard, platform-admin
- [ ] Set up baseURL per project (localhost ports: 5173, 5174, 5175)
- [ ] Configure test directory structure: `tests/e2e/{app-name}/`
- [ ] Add retries (2 for CI, 0 for local)
- [ ] Configure screenshot/video on failure
- [ ] Add `npm run test:e2e` script to root package.json
- [ ] Typecheck passes

#### US-002: Create Test Database Seeding
**Description:** As a developer, I need a consistent test database state so tests are reproducible.

**Acceptance Criteria:**
- [ ] Create `tests/fixtures/seed.sql` with test data
- [ ] Include test organization: "OFFLEASH Demo" (slug: offleash-demo)
- [ ] Include test users: admin@test.com, walker@test.com, customer@test.com (password: Test123!)
- [ ] Include test services (30min walk, 60min walk, pet sitting)
- [ ] Include test locations (2 addresses with lat/long)
- [ ] Create `scripts/reset-test-db.sh` to reset and seed
- [ ] Document test credentials in `tests/README.md`

#### US-003: Configure GitHub Actions CI Pipeline
**Description:** As a developer, I need tests to run automatically on PR and push so regressions are caught early.

**Acceptance Criteria:**
- [ ] Create `.github/workflows/e2e-tests.yml`
- [ ] Run on: push to main, PR to main
- [ ] Start API server (Rust) in background
- [ ] Start all three web apps in background
- [ ] Run database migrations and seed
- [ ] Execute Playwright tests with sharding (4 shards)
- [ ] Upload test artifacts (screenshots, videos, traces) on failure
- [ ] Report test results in PR comments
- [ ] Typecheck passes

#### US-004: Create Test Utilities and Helpers
**Description:** As a developer, I need shared test utilities so I can write tests efficiently.

**Acceptance Criteria:**
- [ ] Create `tests/utils/auth.ts` with login helpers
- [ ] Create `tests/utils/api.ts` with API request helpers
- [ ] Create `tests/utils/fixtures.ts` with test data factories
- [ ] Create `tests/utils/database.ts` with DB query helpers
- [ ] Add `loginAsCustomer()`, `loginAsWalker()`, `loginAsAdmin()`, `loginAsPlatformAdmin()` helpers
- [ ] Add `createTestBooking()`, `createTestService()` factories
- [ ] Typecheck passes

---

### Phase 2: API Tests (Critical Path)

#### US-005: Authentication API Tests
**Description:** As a developer, I need API tests for all auth endpoints so I can verify authentication works correctly.

**Acceptance Criteria:**
- [ ] Test POST `/auth/login/universal` - valid credentials
- [ ] Test POST `/auth/login/universal` - invalid password (401)
- [ ] Test POST `/auth/login/universal` - unknown email (401)
- [ ] Test POST `/auth/register` - new user creation
- [ ] Test POST `/auth/register` - duplicate email (409)
- [ ] Test GET `/auth/session` - returns user info with valid token
- [ ] Test GET `/auth/session` - returns 401 without token
- [ ] Test POST `/contexts/switch` - switches organization context
- [ ] Test POST `/contexts/switch` - fails for non-member org (403)
- [ ] Test token expiration handling
- [ ] All tests pass in CI

#### US-006: Booking API Tests
**Description:** As a developer, I need API tests for booking CRUD so booking functionality is verified.

**Acceptance Criteria:**
- [ ] Test POST `/bookings` - create new booking as customer
- [ ] Test POST `/bookings` - fails without required fields (400)
- [ ] Test GET `/bookings` - admin sees all org bookings
- [ ] Test GET `/bookings/walker` - walker sees only assigned bookings
- [ ] Test GET `/bookings/:id` - returns booking details
- [ ] Test PUT `/bookings/:id` - update booking status
- [ ] Test POST `/bookings/:id/cancel` - cancels booking
- [ ] Test booking status transitions (pending → confirmed → in_progress → completed)
- [ ] Test booking with invalid walker_id (400)
- [ ] Test booking with conflicting time slot (409)
- [ ] All tests pass in CI

#### US-007: Recurring Booking API Tests
**Description:** As a developer, I need API tests for recurring bookings so subscription-style bookings work.

**Acceptance Criteria:**
- [ ] Test POST `/bookings/recurring` - create weekly series
- [ ] Test POST `/bookings/recurring` - create bi-weekly series
- [ ] Test POST `/bookings/recurring` - create monthly series
- [ ] Test POST `/bookings/recurring/preview` - returns generated dates
- [ ] Test conflict detection during preview
- [ ] Test GET `/bookings/recurring` - list customer's series
- [ ] Test GET `/bookings/recurring/:id` - series with all bookings
- [ ] Test POST `/bookings/recurring/:id/cancel` - cancel future occurrences
- [ ] Test individual booking cancellation doesn't affect series
- [ ] All tests pass in CI

#### US-008: Services API Tests
**Description:** As a developer, I need API tests for service management.

**Acceptance Criteria:**
- [ ] Test GET `/services` - list active services (requires auth)
- [ ] Test POST `/services` - create service (admin only)
- [ ] Test PUT `/services/:id` - update service (admin only)
- [ ] Test DELETE `/services/:id` - deactivate service
- [ ] Test customer cannot create/update services (403)
- [ ] Test price stored correctly in cents
- [ ] All tests pass in CI

#### US-009: Availability API Tests
**Description:** As a developer, I need API tests for availability calculation.

**Acceptance Criteria:**
- [ ] Test GET `/availability/slots` - returns available time slots
- [ ] Test slots respect walker working hours
- [ ] Test slots exclude existing bookings
- [ ] Test slots exclude calendar blocks
- [ ] Test slots respect service duration
- [ ] Test availability for specific walker
- [ ] Test availability across date range
- [ ] All tests pass in CI

#### US-010: Calendar API Tests
**Description:** As a developer, I need API tests for calendar management.

**Acceptance Criteria:**
- [ ] Test GET `/calendar/events` - list events for date range
- [ ] Test POST `/calendar/events` - create block event
- [ ] Test PUT `/calendar/events/:id` - update event
- [ ] Test DELETE `/calendar/events/:id` - delete event
- [ ] Test recurring block creation
- [ ] Test events filter by event_type
- [ ] Test calendar respects user ownership
- [ ] All tests pass in CI

#### US-011: User & Walker Management API Tests
**Description:** As a developer, I need API tests for user and walker operations.

**Acceptance Criteria:**
- [ ] Test GET `/users` - admin lists org users
- [ ] Test POST `/admin/users/walker` - create walker (admin only)
- [ ] Test GET `/users/:id` - get user details
- [ ] Test PUT `/users/:id` - update profile
- [ ] Test GET `/working-hours/:walker_id` - get working hours
- [ ] Test PUT `/working-hours/:walker_id` - update working hours
- [ ] Test GET `/walker-profiles/:id` - get walker profile
- [ ] Test PUT `/walker-profiles/:id` - update walker bio/certifications
- [ ] All tests pass in CI

#### US-012: Payment API Tests (Mocked)
**Description:** As a developer, I need API tests for payment flows with mocked providers.

**Acceptance Criteria:**
- [ ] Create mock Stripe/Square HTTP interceptor
- [ ] Test POST `/checkout/session` - creates payment intent
- [ ] Test GET `/payment-methods` - list saved methods
- [ ] Test POST `/payment-methods` - add new card (mocked)
- [ ] Test DELETE `/payment-methods/:id` - remove card
- [ ] Test checkout calculates fees correctly
- [ ] Test checkout applies tax based on location
- [ ] Test transaction records created on success
- [ ] All tests pass in CI

#### US-013: Multi-Tenant Isolation API Tests
**Description:** As a developer, I need tests verifying data isolation between organizations.

**Acceptance Criteria:**
- [ ] Create two test organizations (org-a, org-b)
- [ ] Test user in org-a cannot see org-b bookings
- [ ] Test user in org-a cannot see org-b services
- [ ] Test user in org-a cannot see org-b users
- [ ] Test context switch only works for member orgs
- [ ] Test API returns 403 for cross-org resource access
- [ ] Test bookings scoped to organization
- [ ] All tests pass in CI

---

### Phase 3: Customer Web E2E Tests

#### US-014: Customer Authentication E2E
**Description:** As a tester, I need E2E tests for customer login/logout flows.

**Acceptance Criteria:**
- [ ] Test successful login with email/password
- [ ] Test login error message for invalid credentials
- [ ] Test redirect to requested page after login
- [ ] Test logout clears session and redirects to login
- [ ] Test protected routes redirect unauthenticated users
- [ ] Test session persists across page navigation
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-015: Customer Booking Flow E2E
**Description:** As a tester, I need E2E tests for the complete booking journey.

**Acceptance Criteria:**
- [ ] Test browse services page displays available services
- [ ] Test service selection navigates to booking form
- [ ] Test location selection (new location or saved)
- [ ] Test date/time picker shows available slots
- [ ] Test walker selection (if multiple available)
- [ ] Test booking summary displays correct info
- [ ] Test booking confirmation creates booking
- [ ] Test booking appears in "My Bookings" list
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-016: Customer Recurring Booking E2E
**Description:** As a tester, I need E2E tests for recurring booking creation.

**Acceptance Criteria:**
- [ ] Test recurring booking option visible in booking flow
- [ ] Test frequency selection (weekly, bi-weekly, monthly)
- [ ] Test end condition selection (# of occurrences vs end date)
- [ ] Test preview shows generated dates
- [ ] Test conflict warning displayed when applicable
- [ ] Test recurring series created successfully
- [ ] Test recurring bookings page lists series
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-017: Customer Profile & Settings E2E
**Description:** As a tester, I need E2E tests for customer account management.

**Acceptance Criteria:**
- [ ] Test view profile information
- [ ] Test update profile (name, phone, timezone)
- [ ] Test view saved locations
- [ ] Test add new location
- [ ] Test edit existing location
- [ ] Test delete location
- [ ] Test view payment methods
- [ ] Test add payment method (with mock)
- [ ] Test set default payment method
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-018: Customer Booking Management E2E
**Description:** As a tester, I need E2E tests for viewing and managing bookings.

**Acceptance Criteria:**
- [ ] Test bookings list shows upcoming bookings
- [ ] Test bookings list shows past bookings (separate tab/filter)
- [ ] Test booking detail page displays all info
- [ ] Test cancel booking flow with confirmation
- [ ] Test cancelled booking shows in list with status
- [ ] Test booking status filter works
- [ ] Verify in browser
- [ ] All tests pass in CI

---

### Phase 4: Admin Dashboard E2E Tests

#### US-019: Admin Authentication E2E
**Description:** As a tester, I need E2E tests for admin login with role verification.

**Acceptance Criteria:**
- [ ] Test admin login with valid staff credentials
- [ ] Test customer-role user cannot access admin (redirect/error)
- [ ] Test walker-role user has limited access
- [ ] Test owner-role user has full access
- [ ] Test context switching between organizations
- [ ] Test logout clears admin session
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-020: Admin Dashboard Overview E2E
**Description:** As a tester, I need E2E tests for admin dashboard metrics.

**Acceptance Criteria:**
- [ ] Test dashboard loads with booking counts
- [ ] Test walker view shows limited metrics (no revenue)
- [ ] Test admin view shows full metrics
- [ ] Test recent bookings list displays
- [ ] Test upcoming bookings list displays
- [ ] Test navigation to booking details
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-021: Admin Bookings Management E2E
**Description:** As a tester, I need E2E tests for admin booking operations.

**Acceptance Criteria:**
- [ ] Test bookings list shows all org bookings
- [ ] Test status filter functionality
- [ ] Test search by customer/walker name
- [ ] Test booking detail view
- [ ] Test update booking status (confirm, start, complete)
- [ ] Test cancel booking with reason
- [ ] Test assign/reassign walker
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-022: Admin Calendar E2E
**Description:** As a tester, I need E2E tests for calendar management.

**Acceptance Criteria:**
- [ ] Test calendar week view displays
- [ ] Test navigate between weeks
- [ ] Test walker filter dropdown
- [ ] Test bookings appear on calendar
- [ ] Test create time block (single)
- [ ] Test create recurring block
- [ ] Test delete block
- [ ] Test working hours displayed as background
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-023: Admin Walker Management E2E
**Description:** As a tester, I need E2E tests for walker CRUD operations.

**Acceptance Criteria:**
- [ ] Test walkers list displays all walkers
- [ ] Test create new walker (invite flow)
- [ ] Test view walker detail page
- [ ] Test edit working hours
- [ ] Test toggle walker active status
- [ ] Test walker stats display (bookings count)
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-024: Admin Services Management E2E
**Description:** As a tester, I need E2E tests for service CRUD.

**Acceptance Criteria:**
- [ ] Test services list displays
- [ ] Test create new service
- [ ] Test edit service (name, price, duration)
- [ ] Test toggle service active/inactive
- [ ] Test price displays correctly (dollars, not cents)
- [ ] Verify in browser
- [ ] All tests pass in CI

---

### Phase 5: Platform Admin E2E Tests

#### US-025: Platform Admin Authentication E2E
**Description:** As a tester, I need E2E tests for platform admin access.

**Acceptance Criteria:**
- [ ] Test platform admin login with platform token
- [ ] Test regular user cannot access platform admin
- [ ] Test platform admin session separate from org sessions
- [ ] Test logout clears platform session only
- [ ] Verify in browser
- [ ] All tests pass in CI

#### US-026: Tenant Management E2E
**Description:** As a tester, I need E2E tests for tenant provisioning.

**Acceptance Criteria:**
- [ ] Test tenants list displays all organizations
- [ ] Test create new tenant form
- [ ] Test tenant creation provisions database
- [ ] Test assign initial admin to new tenant
- [ ] Test tenant detail page shows info
- [ ] Test tenant slug is unique
- [ ] Verify in browser
- [ ] All tests pass in CI

---

### Phase 6: iOS App Tests

#### US-027: iOS Test Infrastructure Setup
**Description:** As a developer, I need iOS UI testing configured with XCTest.

**Acceptance Criteria:**
- [ ] Create `OFFLEASHUITests` target in Xcode project
- [ ] Configure test scheme for UI tests
- [ ] Set up test environment variables (API URL, test credentials)
- [ ] Create `TestHelpers.swift` with common utilities
- [ ] Add login/logout helper methods
- [ ] Configure CI to run iOS tests (xcodebuild)
- [ ] Typecheck/build passes

#### US-028: iOS Authentication Tests
**Description:** As a tester, I need iOS tests for login/logout flows.

**Acceptance Criteria:**
- [ ] Test login screen displays
- [ ] Test successful login with email/password
- [ ] Test error display for invalid credentials
- [ ] Test logout clears session
- [ ] Test biometric authentication (if enabled)
- [ ] Test session persistence after app restart
- [ ] All tests pass in CI

#### US-029: iOS Booking Flow Tests
**Description:** As a tester, I need iOS tests for the booking journey.

**Acceptance Criteria:**
- [ ] Test services list loads
- [ ] Test service selection
- [ ] Test location selection/creation
- [ ] Test date/time picker interaction
- [ ] Test booking confirmation
- [ ] Test booking appears in list
- [ ] All tests pass in CI

#### US-030: iOS Customer Profile Tests
**Description:** As a tester, I need iOS tests for profile management.

**Acceptance Criteria:**
- [ ] Test profile view displays user info
- [ ] Test profile edit and save
- [ ] Test locations list view
- [ ] Test add/edit location
- [ ] Test payment methods display
- [ ] All tests pass in CI

---

### Phase 7: Optional Real Payment Provider Tests

#### US-031: Stripe Test Mode Integration Tests
**Description:** As a developer, I need optional tests against Stripe test mode for payment verification.

**Acceptance Criteria:**
- [ ] Test only runs when STRIPE_TEST_KEY env var is set
- [ ] Test create payment intent with test card
- [ ] Test payment method attachment
- [ ] Test successful charge with test card 4242424242424242
- [ ] Test declined card handling (4000000000000002)
- [ ] Test 3D Secure flow (4000002760003184)
- [ ] Test refund processing
- [ ] Document skip behavior when keys not configured
- [ ] All tests pass when configured

---

### Phase 8: Staging Environment Validation

#### US-032: Staging Smoke Tests
**Description:** As a developer, I need smoke tests that run against staging environment.

**Acceptance Criteria:**
- [ ] Create `tests/staging/smoke.spec.ts`
- [ ] Test health endpoint responds
- [ ] Test login page loads
- [ ] Test can authenticate (staging test user)
- [ ] Test services page loads
- [ ] Test bookings page loads
- [ ] Test admin dashboard loads (with admin user)
- [ ] Configure to run via `npm run test:staging`
- [ ] Add to deployment pipeline after staging deploy
- [ ] All tests pass against staging

---

## Functional Requirements

### Test Infrastructure
- FR-1: Playwright must be configured with separate projects for each web app
- FR-2: Test database must be seedable with consistent fixture data
- FR-3: GitHub Actions must run all tests on PR and main branch pushes
- FR-4: Test artifacts (screenshots, videos, traces) must be uploaded on failure
- FR-5: Tests must support parallel execution with sharding

### API Tests
- FR-6: All API endpoints must have corresponding test coverage
- FR-7: API tests must verify both success and error responses
- FR-8: API tests must verify HTTP status codes and response bodies
- FR-9: Authentication tests must cover token lifecycle (create, refresh, expire)
- FR-10: Multi-tenant tests must verify data isolation between organizations

### E2E Tests
- FR-11: E2E tests must cover complete user journeys (not just individual pages)
- FR-12: E2E tests must use realistic test data from seeded database
- FR-13: E2E tests must clean up created data after each test (or use isolated transactions)
- FR-14: E2E tests must handle async operations with proper waits (no arbitrary sleeps)
- FR-15: E2E tests must work across all supported browsers (Chromium, Firefox, WebKit)

### Payment Tests
- FR-16: Payment tests must use mocked providers by default
- FR-17: Real payment provider tests must be opt-in via environment variables
- FR-18: Mock payment responses must match real provider response structures

### iOS Tests
- FR-19: iOS tests must use XCTest UI testing framework
- FR-20: iOS tests must run in CI via xcodebuild command
- FR-21: iOS tests must support both simulator and device testing

### Environment Support
- FR-22: Tests must support local development environment (localhost)
- FR-23: Tests must support CI environment (GitHub Actions)
- FR-24: Staging smoke tests must run against deployed staging URL

---

## Non-Goals (Out of Scope)

- Performance/load testing (separate initiative)
- Visual regression testing (screenshot comparison)
- Accessibility testing (a11y audits)
- Mobile browser testing (responsive design)
- Production environment testing (security risk)
- Third-party integration testing beyond payment (e.g., Google Calendar sync)
- Internationalization (i18n) testing
- Offline/PWA functionality testing
- Push notification testing

---

## Technical Considerations

### Test Stack
- **Web E2E:** Playwright 1.57+
- **iOS UI:** XCTest with XCUITest
- **API Tests:** Playwright API testing (same framework for consistency)
- **Mocking:** Playwright route interception for HTTP mocks
- **CI:** GitHub Actions with ubuntu-latest runners

### Database Strategy
- Use PostgreSQL with test-specific database
- Reset and seed before each test run (not each test)
- Use transactions for isolation where possible
- Test data should be minimal but representative

### Authentication in Tests
- Store test tokens in Playwright storage state
- Reuse authenticated state across tests in same suite
- Create helper functions for each role (customer, walker, admin, platform-admin)

### Parallel Execution
- Playwright sharding (4 workers in CI)
- Isolate test data by test file where possible
- Use unique identifiers for created resources

### Environment Variables
```bash
# Required
DATABASE_URL=postgresql://...
JWT_SECRET=test-secret
PUBLIC_API_URL=http://localhost:8080

# Optional (for real payment tests)
STRIPE_TEST_SECRET_KEY=sk_test_...
STRIPE_TEST_PUBLISHABLE_KEY=pk_test_...

# Staging
STAGING_URL=https://offleash.world
STAGING_API_URL=https://api.offleash.world
STAGING_TEST_USER=staging-test@offleash.world
STAGING_TEST_PASSWORD=...
```

### File Structure
```
/tests/
├── e2e/
│   ├── customer-web/
│   │   ├── auth.spec.ts
│   │   ├── booking.spec.ts
│   │   ├── recurring-booking.spec.ts
│   │   ├── profile.spec.ts
│   │   └── booking-management.spec.ts
│   ├── admin-dashboard/
│   │   ├── auth.spec.ts
│   │   ├── dashboard.spec.ts
│   │   ├── bookings.spec.ts
│   │   ├── calendar.spec.ts
│   │   ├── walkers.spec.ts
│   │   └── services.spec.ts
│   └── platform-admin/
│       ├── auth.spec.ts
│       └── tenants.spec.ts
├── api/
│   ├── auth.spec.ts
│   ├── bookings.spec.ts
│   ├── recurring-bookings.spec.ts
│   ├── services.spec.ts
│   ├── availability.spec.ts
│   ├── calendar.spec.ts
│   ├── users.spec.ts
│   ├── payments.spec.ts
│   └── multi-tenant.spec.ts
├── staging/
│   └── smoke.spec.ts
├── fixtures/
│   ├── seed.sql
│   └── test-data.ts
├── utils/
│   ├── auth.ts
│   ├── api.ts
│   ├── database.ts
│   └── fixtures.ts
├── playwright.config.ts
└── README.md

/ralph/apps/ios/OFFLEASHUITests/
├── AuthenticationTests.swift
├── BookingFlowTests.swift
├── ProfileTests.swift
└── TestHelpers.swift
```

---

## Success Metrics

- **Coverage:** 100% of critical path endpoints have API tests
- **Coverage:** All user stories have corresponding E2E tests
- **Reliability:** Tests pass consistently (< 1% flakiness)
- **Speed:** Full test suite completes in < 15 minutes in CI
- **CI Integration:** All PRs blocked on failing tests
- **Regression Prevention:** Zero production bugs in tested flows

---

## Implementation Sequence

### Week 1: Infrastructure
1. US-001: Playwright setup
2. US-002: Test database seeding
3. US-003: GitHub Actions CI
4. US-004: Test utilities

### Week 2: API Tests (Auth & Booking)
5. US-005: Authentication API
6. US-006: Booking API
7. US-007: Recurring Booking API

### Week 3: API Tests (Remaining)
8. US-008: Services API
9. US-009: Availability API
10. US-010: Calendar API
11. US-011: User/Walker API
12. US-012: Payment API (mocked)
13. US-013: Multi-tenant isolation

### Week 4: Customer Web E2E
14. US-014: Customer auth
15. US-015: Booking flow
16. US-016: Recurring booking
17. US-017: Profile/settings
18. US-018: Booking management

### Week 5: Admin Dashboard E2E
19. US-019: Admin auth
20. US-020: Dashboard overview
21. US-021: Bookings management
22. US-022: Calendar
23. US-023: Walker management
24. US-024: Services

### Week 6: Platform Admin & iOS
25. US-025: Platform admin auth
26. US-026: Tenant management
27. US-027: iOS infrastructure
28. US-028: iOS auth
29. US-029: iOS booking
30. US-030: iOS profile

### Week 7: Payment & Staging
31. US-031: Stripe integration tests
32. US-032: Staging smoke tests

---

## Open Questions

1. Should we implement contract testing (Pact) for API versioning?
2. Do we need database snapshot/restore for faster test isolation?
3. Should iOS tests run on every PR or only on iOS-related changes?
4. What is the staging test user provisioning process?
5. Should we add Slack/Discord notifications for test failures in CI?
6. Do we need separate test suites for different user roles (walker-only tests)?

---

## Appendix: Test User Credentials

| Role | Email | Password | Organization |
|------|-------|----------|--------------|
| Customer | customer@test.offleash.world | Test123! | OFFLEASH Demo |
| Walker | walker@test.offleash.world | Test123! | OFFLEASH Demo |
| Admin | admin@test.offleash.world | Test123! | OFFLEASH Demo |
| Owner | owner@test.offleash.world | Test123! | OFFLEASH Demo |
| Platform Admin | platform@test.offleash.world | Test123! | N/A (platform-level) |
| Org-B Customer | customer@org-b.test | Test123! | Test Org B |

## Appendix: Key API Endpoints to Test

| Endpoint | Method | Auth Required | Role |
|----------|--------|---------------|------|
| `/auth/login/universal` | POST | No | Any |
| `/auth/register` | POST | No | Any |
| `/auth/session` | GET | Yes | Any |
| `/contexts/switch` | POST | Yes | Any |
| `/bookings` | GET | Yes | Admin/Owner |
| `/bookings` | POST | Yes | Customer |
| `/bookings/walker` | GET | Yes | Walker |
| `/bookings/recurring` | POST | Yes | Customer |
| `/services` | GET | Yes | Any |
| `/services` | POST | Yes | Admin/Owner |
| `/availability/slots` | GET | Yes | Any |
| `/calendar/events` | GET | Yes | Any |
| `/calendar/events` | POST | Yes | Walker/Admin |
| `/users` | GET | Yes | Admin/Owner |
| `/working-hours/:id` | GET/PUT | Yes | Walker/Admin |
| `/payment-methods` | GET/POST | Yes | Customer |
| `/checkout/session` | POST | Yes | Customer |
