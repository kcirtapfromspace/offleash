# OFFLEASH iOS Appium Tests

Automated E2E tests for the OFFLEASH iOS app using Appium and WebdriverIO.

## Prerequisites

- Node.js 20+
- Xcode with iOS Simulator
- Appium 2.x with XCUITest driver

## Setup

```bash
# Install dependencies
npm install

# Install Appium globally and XCUITest driver
npm install -g appium
appium driver install xcuitest
```

## Running Tests

```bash
# Run all tests
npm test

# Run specific test suite
npm run test:auth
npm run test:customer-booking
npm run test:customer-profile
npm run test:walker-core
npm run test:walker-settings
npm run test:error-handling
npm run test:multi-tenant

# Run critical tests only (fast feedback)
npm run test:critical
```

## Test Suites

| Suite | Tests | Coverage |
|-------|-------|----------|
| auth | 10 | Login, registration, OAuth, session |
| customer-booking | 16 | Services, booking flow, management |
| customer-profile | 12 | Profile, pets, locations, payments |
| walker-core | 14 | Onboarding, dashboard, requests |
| walker-settings | 8 | Calendar, map, settings |
| error-handling | 5 | Network, API, validation errors |
| multi-tenant | 3 | Org switching, branding |

## Configuration

Set environment variables:

```bash
IOS_APP_PATH=/path/to/OFFLEASH.app
IOS_DEVICE_NAME="iPhone 15 Pro"
IOS_VERSION="17.2"
```

## Coverage Check

Verify all PRD test IDs are implemented:

```bash
npm run coverage:check
```

## Reporting

Generate Allure report:

```bash
npm run allure:generate
npm run allure:open
```

## CI/CD Integration

Tests run automatically on:
- Pull requests to `main`
- Push to `main` branch

**TestFlight deployments are blocked if tests fail.**
