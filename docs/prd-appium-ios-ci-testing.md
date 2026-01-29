# PRD: Appium iOS CI Testing for OFFLEASH

## Document Info
- **Author**: Engineering Team
- **Created**: 2026-01-28
- **Status**: Draft
- **Version**: 1.0

---

## 1. Overview

### 1.1 Problem Statement
The OFFLEASH iOS app currently has XCUITest-based UI tests, but lacks comprehensive cross-platform mobile testing infrastructure. As the app grows, we need robust, maintainable E2E tests that:
- Run automatically on every PR and before TestFlight deployments
- Block TestFlight publishing when critical tests fail
- Provide consistent test coverage across all app features
- Generate actionable test reports and debugging artifacts

### 1.2 Goals
1. Implement Appium-based iOS testing framework for comprehensive feature coverage
2. Integrate tests into GitHub Actions CI/CD pipeline
3. Configure test failures to block TestFlight deployments
4. Achieve 80%+ coverage of critical user flows
5. Reduce regression bugs in production by 50%

### 1.3 Non-Goals
- Android testing (separate PRD)
- Performance/load testing
- Security penetration testing
- Replacing existing XCUITests (Appium will complement them)

---

## 2. Current State Analysis

### 2.1 Existing Test Infrastructure
- **Framework**: XCUITest (native iOS)
- **Location**: `apps/ios/OFFLEASHUITests/`
- **Test Suites**:
  - `E2ETests.swift` - 18 test methods covering registration, login, services, booking
  - `WalkerAppDemoTests.swift` - 7 test methods for walker flows
- **CI Integration**: GitHub Actions with Fastlane (`ui_test` lane)

### 2.2 Current CI/CD Pipeline
```
PR → Build → (Tests Optional) → Merge
Main Branch → Build → Deploy to TestFlight (no test gate)
```

### 2.3 Gaps
- Tests do not block TestFlight deployment
- Limited walker flow coverage
- No payment flow testing
- No multi-tenant/org switching tests
- No network failure simulation
- No accessibility testing

---

## 3. Technical Architecture

### 3.1 Technology Stack
| Component | Technology | Purpose |
|-----------|------------|---------|
| Test Framework | Appium 2.x | Cross-platform mobile automation |
| Test Runner | WebdriverIO | JavaScript test execution |
| Language | TypeScript | Type-safe test scripts |
| Assertions | Chai/Expect | Test assertions |
| Reporting | Allure | Test reports and artifacts |
| CI/CD | GitHub Actions | Automation pipeline |
| Simulator | Xcode Simulator | iOS test execution |

### 3.2 Project Structure
```
apps/ios/appium-tests/
├── config/
│   ├── wdio.conf.ts           # WebdriverIO configuration
│   ├── capabilities.ts        # iOS simulator capabilities
│   └── environments.ts        # Test environment configs
├── tests/
│   ├── auth/
│   │   ├── login.spec.ts
│   │   ├── registration.spec.ts
│   │   ├── oauth.spec.ts
│   │   └── session.spec.ts
│   ├── customer/
│   │   ├── services.spec.ts
│   │   ├── booking-flow.spec.ts
│   │   ├── recurring-booking.spec.ts
│   │   ├── profile.spec.ts
│   │   ├── pets.spec.ts
│   │   ├── locations.spec.ts
│   │   └── payments.spec.ts
│   ├── walker/
│   │   ├── onboarding.spec.ts
│   │   ├── dashboard.spec.ts
│   │   ├── booking-requests.spec.ts
│   │   ├── calendar.spec.ts
│   │   ├── map.spec.ts
│   │   └── settings.spec.ts
│   ├── multi-tenant/
│   │   ├── org-switching.spec.ts
│   │   └── branding.spec.ts
│   └── error-handling/
│       ├── network-failures.spec.ts
│       └── validation-errors.spec.ts
├── pages/
│   ├── auth/
│   │   ├── LoginPage.ts
│   │   ├── RegisterPage.ts
│   │   └── RoleSelectionPage.ts
│   ├── customer/
│   │   ├── ServicesPage.ts
│   │   ├── BookingFlowPage.ts
│   │   ├── ProfilePage.ts
│   │   └── ...
│   └── walker/
│       ├── DashboardPage.ts
│       ├── CalendarPage.ts
│       └── ...
├── utils/
│   ├── helpers.ts
│   ├── test-data.ts
│   └── api-helpers.ts
├── package.json
├── tsconfig.json
└── README.md
```

### 3.3 Appium Capabilities
```typescript
// config/capabilities.ts
export const iosCapabilities = {
  platformName: 'iOS',
  'appium:automationName': 'XCUITest',
  'appium:deviceName': 'iPhone 15 Pro',
  'appium:platformVersion': '17.2',
  'appium:app': process.env.IOS_APP_PATH || './build/OFFLEASH.app',
  'appium:bundleId': 'com.offleash.ios',
  'appium:noReset': false,
  'appium:fullReset': false,
  'appium:wdaLaunchTimeout': 120000,
  'appium:wdaConnectionTimeout': 120000,
  'appium:newCommandTimeout': 300,
  'appium:screenshotQuality': 2,
  'appium:mjpegScreenshotUrl': true,
};
```

---

## 4. Test Coverage Requirements

### 4.1 Authentication & Onboarding (Priority: Critical)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| AUTH-001 | Role Selection | Launch app → Select Customer | Navigate to login |
| AUTH-002 | Role Selection Walker | Launch app → Select Walker | Navigate to login |
| AUTH-003 | Email Login Success | Enter valid email/password → Submit | Dashboard loads |
| AUTH-004 | Email Login Invalid | Enter invalid credentials → Submit | Error message shown |
| AUTH-005 | Password Validation | Enter weak password | Strength indicator shows |
| AUTH-006 | Registration Flow | Fill form → Submit → Verify email | Account created |
| AUTH-007 | Google OAuth | Tap Google Sign-In → Complete OAuth | Dashboard loads |
| AUTH-008 | Session Persistence | Login → Kill app → Relaunch | Still authenticated |
| AUTH-009 | Session Expiration | Wait for token expiry | Redirect to login |
| AUTH-010 | Logout | Tap logout → Confirm | Return to role selection |

### 4.2 Customer - Services (Priority: Critical)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| SVC-001 | Services Load | Navigate to Services tab | Service list displays |
| SVC-002 | Service Details | Tap service card | Details shown (price, duration) |
| SVC-003 | Pull to Refresh | Pull down on list | List refreshes |
| SVC-004 | Empty State | No services available | Empty state message |
| SVC-005 | Loading State | Slow network | Loading indicator shown |
| SVC-006 | Error State | Network failure | Error message with retry |

### 4.3 Customer - Booking Flow (Priority: Critical)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| BKG-001 | Start Booking | Select service → Tap Book | Booking flow opens |
| BKG-002 | Location Selection | Select existing location | Proceed to date/time |
| BKG-003 | Add New Location | Tap Add Location → Fill form | Location saved |
| BKG-004 | Date Selection | Select available date | Time slots shown |
| BKG-005 | Time Selection | Select available slot | Proceed to review |
| BKG-006 | Review Order | Verify details | Correct info displayed |
| BKG-007 | Complete Booking | Tap Confirm | Confirmation shown |
| BKG-008 | Recurring Booking | Enable recurring → Set frequency | Multiple bookings created |
| BKG-009 | Add Notes | Enter special instructions | Notes saved with booking |
| BKG-010 | Cancel Before Submit | Tap Cancel | Return to services |

### 4.4 Customer - Booking Management (Priority: High)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| BMG-001 | View Bookings | Navigate to Bookings tab | Booking list displays |
| BMG-002 | Booking Details | Tap booking | Full details shown |
| BMG-003 | Reschedule Booking | Tap Reschedule → New time | Booking updated |
| BMG-004 | Cancel Booking | Tap Cancel → Confirm | Booking cancelled |
| BMG-005 | Filter by Status | Select status filter | Filtered list shown |
| BMG-006 | Empty State | No bookings | Empty state message |

### 4.5 Customer - Profile & Settings (Priority: High)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| PRF-001 | View Profile | Navigate to Profile tab | Profile info shown |
| PRF-002 | Edit Profile | Update name → Save | Changes persisted |
| PRF-003 | Add Pet | Tap Add Pet → Fill form | Pet added |
| PRF-004 | Edit Pet | Select pet → Update → Save | Pet updated |
| PRF-005 | Delete Pet | Select pet → Delete → Confirm | Pet removed |
| PRF-006 | View Locations | Tap My Locations | Location list shown |
| PRF-007 | Add Location | Tap Add → Fill address | Location saved |
| PRF-008 | Set Default Location | Tap Set Default | Location marked default |

### 4.6 Customer - Payments (Priority: High)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| PAY-001 | View Payment Methods | Navigate to Payments | Methods listed |
| PAY-002 | Add Card | Tap Add → Enter details | Card saved |
| PAY-003 | Remove Card | Tap Remove → Confirm | Card deleted |
| PAY-004 | Transaction History | Tap History | Transactions listed |
| PAY-005 | Subscription View | Tap Subscriptions | Active subs shown |

### 4.7 Walker - Onboarding (Priority: Critical)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WON-001 | Create Organization | Select Create → Fill details | Org created |
| WON-002 | Join Organization | Enter invite code → Submit | Joined org |
| WON-003 | Invalid Invite | Enter invalid code | Error shown |
| WON-004 | Complete Onboarding | Finish all steps | Dashboard access |

### 4.8 Walker - Dashboard (Priority: Critical)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WDB-001 | Dashboard Load | Login as walker | Stats displayed |
| WDB-002 | Today's Bookings | View count | Correct count shown |
| WDB-003 | Toggle On-Duty | Tap toggle | Status changes |
| WDB-004 | Weekly Earnings | View earnings | Correct amount shown |
| WDB-005 | Performance Metrics | View metrics | Metrics displayed |

### 4.9 Walker - Booking Requests (Priority: Critical)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WBR-001 | View Requests | Navigate to Requests | Pending list shown |
| WBR-002 | Accept Request | Tap Accept | Booking confirmed |
| WBR-003 | Decline Request | Tap Decline → Reason | Request declined |
| WBR-004 | Request Details | Tap request | Full details shown |
| WBR-005 | Empty State | No requests | Empty message shown |

### 4.10 Walker - Calendar & Map (Priority: High)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WCL-001 | Calendar Month View | Navigate to Calendar | Month view shown |
| WCL-002 | Calendar Week View | Toggle to week | Week view shown |
| WCL-003 | Select Date | Tap date with booking | Bookings listed |
| WMP-001 | Map Load | Navigate to Map | Map displays |
| WMP-002 | Location Permission | First access | Permission prompt |
| WMP-003 | Booking Markers | View map | Markers shown |

### 4.11 Walker - Settings (Priority: Medium)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WST-001 | Working Hours | Set availability | Hours saved |
| WST-002 | Service Areas | Define areas | Areas saved |
| WST-003 | Profile Update | Edit profile | Changes saved |
| WST-004 | Invite Walker | Send invite | Invite sent |
| WST-005 | Invite Customer | Send invite | Invite sent |

### 4.12 Multi-Tenant (Priority: Medium)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| MTN-001 | Org Switching | Switch organization | Context changes |
| MTN-002 | Branding Applied | Switch org | Branding updates |
| MTN-003 | Role Per Org | Check permissions | Correct role active |

### 4.13 Error Handling (Priority: High)

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| ERR-001 | Network Offline | Disable network | Offline message |
| ERR-002 | API Timeout | Slow response | Timeout error shown |
| ERR-003 | 401 Unauthorized | Invalid token | Redirect to login |
| ERR-004 | 500 Server Error | API failure | Error with retry |
| ERR-005 | Validation Error | Invalid input | Field errors shown |

---

## 5. CI/CD Integration

### 5.1 Updated Pipeline Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                         PR Pipeline                              │
├─────────────────────────────────────────────────────────────────┤
│  Build iOS App → Run Appium Tests → Report Results              │
│       │                │                   │                     │
│       ▼                ▼                   ▼                     │
│   Compile App    Execute Tests      Generate Allure             │
│                  (Parallel by        Report + Upload            │
│                   test suite)        Screenshots                │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    Main Branch Pipeline                          │
├─────────────────────────────────────────────────────────────────┤
│  Build → Appium Tests → ┬─ Pass → Deploy TestFlight             │
│                         │                                        │
│                         └─ Fail → Block + Notify                │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 GitHub Actions Workflow
```yaml
# .github/workflows/ios-appium-tests.yml
name: iOS Appium Tests

on:
  pull_request:
    paths:
      - 'apps/ios/**'
  push:
    branches: [main]
  workflow_dispatch:

env:
  XCODE_VERSION: '15.2'
  IOS_SIMULATOR: 'iPhone 15 Pro'
  IOS_VERSION: '17.2'

jobs:
  build:
    name: Build iOS App
    runs-on: macos-14
    outputs:
      app-path: ${{ steps.build.outputs.app-path }}
    steps:
      - uses: actions/checkout@v4

      - name: Select Xcode
        run: sudo xcode-select -s /Applications/Xcode_${{ env.XCODE_VERSION }}.app

      - name: Cache DerivedData
        uses: actions/cache@v4
        with:
          path: apps/ios/build/DerivedData
          key: ${{ runner.os }}-deriveddata-${{ hashFiles('apps/ios/**/*.swift') }}

      - name: Build for Testing
        id: build
        working-directory: apps/ios
        run: |
          xcodebuild build-for-testing \
            -scheme OFFLEASH \
            -sdk iphonesimulator \
            -destination "platform=iOS Simulator,name=${{ env.IOS_SIMULATOR }},OS=${{ env.IOS_VERSION }}" \
            -derivedDataPath build/DerivedData \
            CODE_SIGNING_ALLOWED=NO

          APP_PATH=$(find build/DerivedData -name "*.app" -type d | head -1)
          echo "app-path=$APP_PATH" >> $GITHUB_OUTPUT

      - name: Upload App Artifact
        uses: actions/upload-artifact@v4
        with:
          name: ios-app
          path: ${{ steps.build.outputs.app-path }}
          retention-days: 1

  appium-tests:
    name: Appium Tests
    needs: build
    runs-on: macos-14
    strategy:
      fail-fast: false
      matrix:
        test-suite:
          - auth
          - customer-booking
          - customer-profile
          - walker-core
          - walker-settings
          - error-handling
    steps:
      - uses: actions/checkout@v4

      - name: Download App
        uses: actions/download-artifact@v4
        with:
          name: ios-app
          path: apps/ios/build

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: apps/ios/appium-tests/package-lock.json

      - name: Install Dependencies
        working-directory: apps/ios/appium-tests
        run: npm ci

      - name: Start Appium Server
        run: |
          npm install -g appium
          appium driver install xcuitest
          appium &
          sleep 10

      - name: Boot Simulator
        run: |
          xcrun simctl boot "${{ env.IOS_SIMULATOR }}" || true
          xcrun simctl bootstatus "${{ env.IOS_SIMULATOR }}" -b

      - name: Run Appium Tests
        working-directory: apps/ios/appium-tests
        env:
          IOS_APP_PATH: ${{ github.workspace }}/apps/ios/build/OFFLEASH.app
          TEST_SUITE: ${{ matrix.test-suite }}
        run: npm run test:${{ matrix.test-suite }}

      - name: Upload Test Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results-${{ matrix.test-suite }}
          path: |
            apps/ios/appium-tests/allure-results
            apps/ios/appium-tests/screenshots
          retention-days: 7

  report:
    name: Generate Report
    needs: appium-tests
    if: always()
    runs-on: ubuntu-latest
    steps:
      - name: Download All Results
        uses: actions/download-artifact@v4
        with:
          pattern: test-results-*
          merge-multiple: true
          path: allure-results

      - name: Generate Allure Report
        uses: simple-elf/allure-report-action@v1.7
        with:
          allure_results: allure-results
          allure_history: allure-history

      - name: Deploy Report to Pages
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: allure-history

  gate-testflight:
    name: TestFlight Gate
    needs: [appium-tests]
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    outputs:
      tests-passed: ${{ steps.check.outputs.passed }}
    steps:
      - name: Check Test Results
        id: check
        run: |
          # All matrix jobs must pass
          if [ "${{ needs.appium-tests.result }}" == "success" ]; then
            echo "passed=true" >> $GITHUB_OUTPUT
          else
            echo "passed=false" >> $GITHUB_OUTPUT
            echo "::error::Appium tests failed - blocking TestFlight deployment"
            exit 1
          fi

  deploy-testflight:
    name: Deploy to TestFlight
    needs: [gate-testflight]
    if: needs.gate-testflight.outputs.tests-passed == 'true'
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4

      - name: Setup Ruby
        uses: ruby/setup-ruby@v1
        with:
          ruby-version: '3.2'
          bundler-cache: true
          working-directory: apps/ios

      - name: Install Fastlane
        working-directory: apps/ios
        run: bundle install

      - name: Setup Signing
        env:
          APP_STORE_CONNECT_API_KEY: ${{ secrets.APP_STORE_CONNECT_API_KEY }}
          APPLE_DISTRIBUTION_CERTIFICATE: ${{ secrets.APPLE_DISTRIBUTION_CERTIFICATE }}
          APPLE_DISTRIBUTION_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_DISTRIBUTION_CERTIFICATE_PASSWORD }}
          APPLE_PROVISIONING_PROFILE: ${{ secrets.APPLE_PROVISIONING_PROFILE }}
        run: |
          # Decode and install certificates
          echo "$APPLE_DISTRIBUTION_CERTIFICATE" | base64 --decode > cert.p12
          security import cert.p12 -P "$APPLE_DISTRIBUTION_CERTIFICATE_PASSWORD" -A -t cert -f pkcs12 -k ~/Library/Keychains/login.keychain-db

          # Install provisioning profile
          mkdir -p ~/Library/MobileDevice/Provisioning\ Profiles
          echo "$APPLE_PROVISIONING_PROFILE" | base64 --decode > ~/Library/MobileDevice/Provisioning\ Profiles/profile.mobileprovision

      - name: Deploy to TestFlight
        working-directory: apps/ios
        env:
          APP_STORE_CONNECT_API_KEY_ID: ${{ secrets.APP_STORE_CONNECT_API_KEY_ID }}
          APP_STORE_CONNECT_API_ISSUER_ID: ${{ secrets.APP_STORE_CONNECT_API_ISSUER_ID }}
        run: bundle exec fastlane beta_manual

      - name: Notify Success
        if: success()
        run: echo "::notice::Successfully deployed to TestFlight"

      - name: Cleanup
        if: always()
        run: rm -f cert.p12
```

### 5.3 Fastlane Updates
Add new lane to `apps/ios/fastlane/Fastfile`:

```ruby
desc "Run Appium tests"
lane :appium_tests do |options|
  suite = options[:suite] || "all"

  Dir.chdir("../appium-tests") do
    sh("npm ci")
    sh("npm run test:#{suite}")
  end
end

desc "Beta deployment with test gate"
lane :beta_gated do
  # Run tests first
  begin
    appium_tests(suite: "critical")
  rescue => e
    UI.user_error!("Tests failed - blocking deployment: #{e.message}")
  end

  # If tests pass, deploy
  beta_manual
end
```

---

## 6. Test Data Management

### 6.1 Test Accounts
```typescript
// utils/test-data.ts
export const testAccounts = {
  customer: {
    email: 'test-customer@offleash.test',
    password: 'TestPass123!',
    firstName: 'Test',
    lastName: 'Customer'
  },
  walker: {
    email: 'test-walker@offleash.test',
    password: 'TestPass123!',
    firstName: 'Test',
    lastName: 'Walker'
  },
  multiOrg: {
    email: 'test-multi@offleash.test',
    password: 'TestPass123!',
    orgs: ['Org A', 'Org B']
  }
};

export const testData = {
  pet: {
    name: 'Test Dog',
    breed: 'Labrador',
    age: 3
  },
  location: {
    name: 'Home',
    address: '123 Test St',
    city: 'San Francisco',
    state: 'CA',
    zip: '94102'
  }
};
```

### 6.2 API Helpers for Test Setup
```typescript
// utils/api-helpers.ts
export async function seedTestData() {
  // Create test accounts via API
  // Create test services
  // Create test bookings
}

export async function cleanupTestData() {
  // Remove test data after test run
}
```

---

## 7. Success Metrics

### 7.1 KPIs
| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Coverage | 100% of PRD test IDs | Test ID coverage gate |
| Test Pass Rate | 95%+ | Passed / Total tests |
| Flaky Test Rate | <5% | Flaky / Total tests |
| Test Execution Time | <20 min | CI job duration |
| Regression Prevention | 50% reduction | Bugs found in prod |
| TestFlight Block Rate | 100% on failure | Blocked deploys / Failed tests |

### 7.2 Reporting Dashboard
- Allure report published to GitHub Pages
- Slack notifications on test failures
- Weekly test health summary

---

## 8. Implementation Plan

### Phase 1: Foundation (Week 1-2)
- [ ] Set up Appium test project structure
- [ ] Configure WebdriverIO with TypeScript
- [ ] Implement Page Object Model for auth screens
- [ ] Create AUTH test suite (10 tests)
- [ ] Set up CI workflow for PR testing

### Phase 2: Customer Flows (Week 3-4)
- [ ] Implement customer page objects
- [ ] Create Services test suite (6 tests)
- [ ] Create Booking Flow test suite (10 tests)
- [ ] Create Booking Management test suite (6 tests)
- [ ] Create Profile test suite (8 tests)
- [ ] Create Payments test suite (5 tests)

### Phase 3: Walker Flows (Week 5-6)
- [ ] Implement walker page objects
- [ ] Create Onboarding test suite (4 tests)
- [ ] Create Dashboard test suite (5 tests)
- [ ] Create Booking Requests test suite (5 tests)
- [ ] Create Calendar/Map test suite (6 tests)
- [ ] Create Settings test suite (5 tests)

### Phase 4: Advanced & Integration (Week 7-8)
- [ ] Create Multi-tenant test suite (3 tests)
- [ ] Create Error Handling test suite (5 tests)
- [ ] Configure TestFlight blocking gate
- [ ] Set up Allure reporting
- [ ] Performance optimization (parallel execution)
- [ ] Documentation and runbooks

---

## 9. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Flaky tests | High | Implement retry logic, wait strategies |
| Simulator instability | Medium | Reset simulator between suites |
| Long test execution | Medium | Parallel execution, test prioritization |
| Test data conflicts | Medium | Isolated test accounts per suite |
| Appium version changes | Low | Pin versions, update quarterly |

---

## 10. Open Questions

1. Should we maintain both XCUITest and Appium, or migrate fully to Appium?
2. What is the test data retention policy for CI artifacts?
3. Should we integrate with a test management tool (TestRail, Zephyr)?
4. What notification channels for test failures (Slack, email, PagerDuty)?

---

## 11. Appendix

### A. Accessibility Identifiers Required
The following accessibility identifiers must be added to iOS views for reliable test automation. This list is expanded to cover all PRD test IDs:

```swift
// Authentication
"role-selection-customer-button"
"role-selection-walker-button"
"login-email-field"
"login-password-field"
"login-submit-button"
"login-google-button"
"register-link"
"register-first-name"
"register-last-name"
"register-email"
"register-password"
"register-submit-button"
"register-verify-banner"
"logout-button"
"password-strength-indicator"
"auth-error-banner"

// Customer
"tab-services"
"tab-bookings"
"tab-profile"
"services-list"
"service-card-{id}"
"service-details"
"book-service-button"
"booking-location-picker"
"booking-add-location"
"booking-date-picker"
"booking-time-slot-{time}"
"booking-review"
"booking-confirm-button"
"booking-notes-field"
"booking-cancel-button"
"bookings-list"
"booking-item-{id}"
"booking-reschedule-button"
"booking-cancel-confirm"
"booking-filter-button"
"booking-filter-options"
"profile-header"
"profile-edit-button"
"profile-save-button"
"pet-add-button"
"pet-item-{name}"
"pet-delete-button"
"pet-delete-confirm"
"profile-locations-button"
"locations-list"
"location-add-button"
"location-default-button"
"payments-tab"
"payment-methods-list"
"payment-add-button"
"payment-remove-button"
"payment-history-button"
"subscriptions-button"
"payment-history-list"
"subscriptions-list"
"empty-state"
"loading-indicator"
"error-banner"
"retry-button"

// Walker
"walker-dashboard"
"on-duty-toggle"
"today-bookings-count"
"weekly-earnings"
"performance-metrics"
"tab-requests"
"pending-requests-list"
"accept-booking-button"
"decline-booking-button"
"decline-reason-field"
"decline-submit"
"request-item-{id}"
"request-detail"
"tab-calendar"
"calendar-view"
"calendar-toggle-week"
"calendar-day-{date}"
"calendar-bookings-list"
"tab-map"
"map-view"
"booking-markers"
"tab-settings"
"settings-working-hours"
"settings-service-areas"
"settings-profile-update"
"settings-invite-walker"
"settings-invite-customer"
"invite-walker-sheet"
"invite-customer-sheet"

// Walker Onboarding
"walker-create-org-button"
"walker-org-name"
"walker-org-submit"
"walker-join-org-button"
"walker-org-invite-code"
"walker-org-join-submit"
"walker-onboarding-complete"

// Multi-tenant
"org-switcher"
"org-item-{name}"
"org-branding-logo"
"org-role-badge"

// Common
"nav-back-button"
"save-button"
"confirm-button"
"toast-message"
"permission-allow-button"

// Error Handling
"offline-banner"
"timeout-banner"
"unauthorized-banner"
"server-error-banner"
"validation-error"
```

### B. Environment Variables
```bash
# Required for CI
IOS_APP_PATH=/path/to/OFFLEASH.app
APPIUM_HOST=localhost
APPIUM_PORT=4723
TEST_ENV=ci
API_BASE_URL=https://api-staging.offleash.com

# Optional
ALLURE_RESULTS_DIR=./allure-results
SCREENSHOT_ON_FAILURE=true
VIDEO_RECORDING=false
```
