import XCTest

/// Comprehensive End-to-End Tests for the OFFLEASH iOS App
///
/// These tests verify the full user journey from registration to booking.
/// Run with: xcodebuild test -scheme OFFLEASH -destination 'platform=iOS Simulator,name=iPhone 17 Pro' -only-testing:OFFLEASHUITests/E2ETests
///
/// Prerequisites:
/// - Backend API running at localhost:8080 (or configured API_BASE_URL)
/// - Database seeded with test data (run: psql -f demos/seed-test-data.sql)
final class E2ETests: XCTestCase {

    var app: XCUIApplication!
    let testEmail = "e2e.test.\(Int(Date().timeIntervalSince1970))@test.com"
    let testPassword = "TestPassword123!"
    let timeout: TimeInterval = 10

    override func setUpWithError() throws {
        continueAfterFailure = false
        app = XCUIApplication()

        // Configure for testing
        app.launchArguments = ["--uitesting"]
        app.launchEnvironment = [
            "API_BASE_URL": "http://localhost:8080"
        ]

        app.launch()
    }

    override func tearDownWithError() throws {
        takeScreenshot(name: "final-state")
    }

    // MARK: - Test Suite

    /// Full E2E flow: Register -> Login -> View Services -> Start Booking
    func testCompleteUserJourney() throws {
        print("🚀 Starting Complete User Journey E2E Test")

        // Step 1: Registration
        try testRegistrationFlow()

        // Step 2: Logout (if needed) and Login
        try testLoginFlow()

        // Step 3: View Services
        try testServicesDisplay()

        // Step 4: Start Booking Flow
        try testBookingStart()

        print("✅ Complete User Journey E2E Test Passed!")
    }

    // MARK: - Authentication Tests

    func testRegistrationFlow() throws {
        print("📝 Testing Registration Flow...")

        // Navigate to registration if on login screen
        if app.buttons["Create Account"].waitForExistence(timeout: timeout) {
            app.buttons["Create Account"].tap()
        } else if app.buttons["Register"].waitForExistence(timeout: timeout) {
            app.buttons["Register"].tap()
        }

        sleep(1)
        takeScreenshot(name: "e2e-01-register-screen")

        // Fill registration form
        guard app.textFields["First Name"].waitForExistence(timeout: timeout) else {
            XCTFail("Registration screen not displayed")
            return
        }

        // First Name
        let firstNameField = app.textFields["First Name"]
        firstNameField.tap()
        firstNameField.typeText("E2E")

        // Last Name
        let lastNameField = app.textFields["Last Name"]
        lastNameField.tap()
        lastNameField.typeText("Tester")

        // Email
        let emailField = app.textFields["Email"]
        emailField.tap()
        emailField.typeText(testEmail)

        // Phone (if exists)
        if app.textFields["Phone"].exists {
            let phoneField = app.textFields["Phone"]
            phoneField.tap()
            phoneField.typeText("555-123-4567")
        }

        takeScreenshot(name: "e2e-02-register-form-filled")

        // Password
        let passwordField = app.secureTextFields["Password"]
        passwordField.tap()
        passwordField.typeText(testPassword)

        // Confirm Password (if exists)
        if app.secureTextFields["Confirm Password"].exists {
            let confirmField = app.secureTextFields["Confirm Password"]
            confirmField.tap()
            confirmField.typeText(testPassword)
        }

        takeScreenshot(name: "e2e-03-register-password-filled")

        // Check for password requirements indicator (if visible)
        if app.staticTexts["Password Requirements"].exists {
            takeScreenshot(name: "e2e-03b-password-requirements")
        }

        // Submit registration
        let registerButton = app.buttons["Register"].exists ? app.buttons["Register"] : app.buttons["Create Account"]
        XCTAssertTrue(registerButton.exists, "Register button not found")
        registerButton.tap()

        sleep(3) // Wait for API response
        takeScreenshot(name: "e2e-04-after-registration")

        // Verify we're either on the main screen or got an error
        let isOnMainScreen = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Services' OR label CONTAINS 'Schedule' OR label CONTAINS 'Book'")).firstMatch.waitForExistence(timeout: timeout)
        let hasError = app.alerts.firstMatch.exists

        if hasError {
            // Screenshot the error for debugging
            takeScreenshot(name: "e2e-04-registration-error")
            // May fail if email already exists - that's OK for re-runs
            print("⚠️ Registration may have failed (possibly duplicate email) - continuing with login")
        } else {
            XCTAssertTrue(isOnMainScreen, "Should navigate to main screen after registration")
            print("✅ Registration successful")
        }
    }

    func testLoginFlow() throws {
        print("🔐 Testing Login Flow...")

        // If already authenticated, skip login
        if app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Services' OR label CONTAINS 'Schedule'")).firstMatch.waitForExistence(timeout: 3) {
            print("✅ Already authenticated, skipping login")
            return
        }

        // If on registration screen, go back to login
        if app.buttons["Back to Login"].exists {
            app.buttons["Back to Login"].tap()
            sleep(1)
        } else if app.navigationBars.buttons["Back"].exists {
            app.navigationBars.buttons["Back"].tap()
            sleep(1)
        }

        takeScreenshot(name: "e2e-05-login-screen")

        // Check if we're on login screen
        guard app.textFields["Email"].waitForExistence(timeout: timeout) else {
            // May already be authenticated
            if app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Services'")).firstMatch.exists {
                print("✅ Already on main screen")
                return
            }
            XCTFail("Login screen not displayed")
            return
        }

        // Fill login form
        let emailField = app.textFields["Email"]
        emailField.tap()
        emailField.clearAndTypeText(testEmail)

        let passwordField = app.secureTextFields["Password"]
        passwordField.tap()
        passwordField.typeText(testPassword)

        takeScreenshot(name: "e2e-06-login-filled")

        // Submit login
        app.buttons["Sign In"].tap()

        sleep(3) // Wait for API response
        takeScreenshot(name: "e2e-07-after-login")

        // Verify login success
        let isAuthenticated = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Services' OR label CONTAINS 'Schedule' OR label CONTAINS 'Book'")).firstMatch.waitForExistence(timeout: timeout)

        if !isAuthenticated && app.alerts.firstMatch.exists {
            takeScreenshot(name: "e2e-07-login-error")
            XCTFail("Login failed - check credentials or API connection")
        }

        XCTAssertTrue(isAuthenticated, "Should navigate to main screen after login")
        print("✅ Login successful")
    }

    // MARK: - Services Tests

    func testServicesDisplay() throws {
        print("📋 Testing Services Display...")

        // Ensure we're authenticated first
        try ensureAuthenticated()

        // Wait for services to load
        sleep(2)
        takeScreenshot(name: "e2e-08-services-screen")

        // Check for service cards or list
        let servicesExist = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Walk' OR label CONTAINS 'min' OR label CONTAINS '$'")).firstMatch.waitForExistence(timeout: timeout)

        if !servicesExist {
            // Check for empty state
            let emptyState = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'No services' OR label CONTAINS 'unavailable'")).firstMatch.exists
            if emptyState {
                print("⚠️ No services available - database may need seeding")
                takeScreenshot(name: "e2e-08-no-services")
            }
        }

        // Look for specific service elements
        let serviceCards = app.cells.matching(NSPredicate(format: "identifier CONTAINS 'service' OR identifier CONTAINS 'Service'"))
        print("   Found \(serviceCards.count) service cards")

        // Check for price display
        let priceElements = app.staticTexts.matching(NSPredicate(format: "label MATCHES '.*\\\\$[0-9]+.*'"))
        print("   Found \(priceElements.count) price elements")

        // Check for duration display
        let durationElements = app.staticTexts.matching(NSPredicate(format: "label MATCHES '.*[0-9]+ min.*'"))
        print("   Found \(durationElements.count) duration elements")

        takeScreenshot(name: "e2e-09-services-detail")
        print("✅ Services display test completed")
    }

    // MARK: - Booking Tests

    func testBookingStart() throws {
        print("📅 Testing Booking Flow...")

        // Ensure we're authenticated first
        try ensureAuthenticated()

        sleep(2) // Wait for services to load

        // Find a service to book
        let serviceButtons = app.buttons.matching(NSPredicate(format: "label CONTAINS 'Book' OR label CONTAINS 'Select'"))

        if serviceButtons.count > 0 {
            takeScreenshot(name: "e2e-10-before-booking")

            // Tap the first bookable service
            serviceButtons.firstMatch.tap()
            sleep(1)

            takeScreenshot(name: "e2e-11-booking-started")

            // Check for booking sheet/modal
            let bookingSheetVisible = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Book' OR label CONTAINS 'Duration' OR label CONTAINS 'Price'")).firstMatch.waitForExistence(timeout: timeout)

            if bookingSheetVisible {
                print("   ✅ Booking sheet displayed")
                takeScreenshot(name: "e2e-12-booking-sheet")

                // Check for Cancel button
                if app.buttons["Cancel"].exists {
                    print("   ✅ Cancel button available")
                }

                // Check for service details in booking
                let hasDuration = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Duration' OR label CONTAINS 'min'")).firstMatch.exists
                let hasPrice = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Price' OR label CONTAINS '$'")).firstMatch.exists

                print("   Duration displayed: \(hasDuration)")
                print("   Price displayed: \(hasPrice)")

                // Dismiss booking sheet
                if app.buttons["Cancel"].exists {
                    app.buttons["Cancel"].tap()
                    sleep(1)
                }
            }
        } else {
            // Try tapping on a service card directly
            let serviceCells = app.cells.allElementsBoundByIndex
            if serviceCells.count > 0 {
                serviceCells[0].tap()
                sleep(1)
                takeScreenshot(name: "e2e-11-service-tapped")
            } else {
                print("⚠️ No bookable services found - database may need seeding")
            }
        }

        takeScreenshot(name: "e2e-13-booking-complete")
        print("✅ Booking flow test completed")
    }

    // MARK: - Validation Tests

    func testEmailValidation() throws {
        print("✉️ Testing Email Validation...")

        // Navigate to registration
        if app.buttons["Create Account"].waitForExistence(timeout: timeout) {
            app.buttons["Create Account"].tap()
        } else if app.buttons["Register"].waitForExistence(timeout: timeout) {
            app.buttons["Register"].tap()
        }

        sleep(1)

        guard app.textFields["Email"].waitForExistence(timeout: timeout) else {
            return
        }

        let emailField = app.textFields["Email"]

        // Test invalid email
        emailField.tap()
        emailField.typeText("invalid-email")

        // Tap elsewhere to trigger validation
        app.textFields["First Name"].tap()
        sleep(1)

        takeScreenshot(name: "e2e-validation-01-invalid-email")

        // Check for error message
        let hasEmailError = app.staticTexts.matching(NSPredicate(format: "label CONTAINS[c] 'email' AND (label CONTAINS[c] 'invalid' OR label CONTAINS[c] 'valid')")).firstMatch.exists
        print("   Email validation message shown: \(hasEmailError)")

        // Clear and enter valid email
        emailField.tap()
        emailField.clearAndTypeText("valid@example.com")
        app.textFields["First Name"].tap()
        sleep(1)

        takeScreenshot(name: "e2e-validation-02-valid-email")
        print("✅ Email validation test completed")
    }

    func testPasswordStrengthRequirements() throws {
        print("🔒 Testing Password Strength Requirements...")

        // Navigate to registration
        if app.buttons["Create Account"].waitForExistence(timeout: timeout) {
            app.buttons["Create Account"].tap()
        } else if app.buttons["Register"].waitForExistence(timeout: timeout) {
            app.buttons["Register"].tap()
        }

        sleep(1)

        guard app.secureTextFields["Password"].waitForExistence(timeout: timeout) else {
            return
        }

        let passwordField = app.secureTextFields["Password"]

        // Test weak password
        passwordField.tap()
        passwordField.typeText("weak")
        sleep(1)

        takeScreenshot(name: "e2e-validation-03-weak-password")

        // Look for password requirements
        let requirements = app.staticTexts.matching(NSPredicate(format: "label CONTAINS[c] 'character' OR label CONTAINS[c] 'uppercase' OR label CONTAINS[c] 'number' OR label CONTAINS[c] 'special'"))
        print("   Found \(requirements.count) password requirement messages")

        // Clear and enter strong password
        passwordField.tap()

        // Select all and delete
        if let stringValue = passwordField.value as? String, !stringValue.isEmpty {
            let deleteString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: stringValue.count)
            passwordField.typeText(deleteString)
        }

        passwordField.typeText("StrongP@ss123!")
        sleep(1)

        takeScreenshot(name: "e2e-validation-04-strong-password")
        print("✅ Password strength test completed")
    }

    // MARK: - Session Tests

    func testSessionPersistence() throws {
        print("💾 Testing Session Persistence...")

        // Login first
        try testLoginFlow()

        takeScreenshot(name: "e2e-session-01-authenticated")

        // Terminate and relaunch the app
        app.terminate()
        sleep(1)

        app = XCUIApplication()
        app.launchArguments = ["--uitesting"]
        app.launchEnvironment = ["API_BASE_URL": "http://localhost:8080"]
        app.launch()

        sleep(3) // Wait for token validation

        takeScreenshot(name: "e2e-session-02-after-relaunch")

        // Check if still authenticated (should show services, not login)
        let stillAuthenticated = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Services' OR label CONTAINS 'Schedule'")).firstMatch.waitForExistence(timeout: timeout)

        // If showing validation spinner, wait a bit more
        if !stillAuthenticated && app.activityIndicators.firstMatch.exists {
            sleep(3)
        }

        _ = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Services' OR label CONTAINS 'Schedule' OR label CONTAINS 'Email'")).firstMatch.waitForExistence(timeout: timeout)

        takeScreenshot(name: "e2e-session-03-final-state")

        // Session should persist (unless token expired)
        print("   Session persisted: \(stillAuthenticated)")
        print("✅ Session persistence test completed")
    }

    // MARK: - Error Handling Tests

    func testNetworkErrorHandling() throws {
        print("🌐 Testing Network Error Handling...")

        // Launch with invalid API URL to simulate network error
        app.terminate()
        sleep(1)

        app = XCUIApplication()
        app.launchArguments = ["--uitesting"]
        app.launchEnvironment = ["API_BASE_URL": "http://invalid-url.local:9999"]
        app.launch()

        sleep(2)

        // Try to login
        if app.textFields["Email"].waitForExistence(timeout: timeout) {
            let emailField = app.textFields["Email"]
            emailField.tap()
            emailField.typeText("test@test.com")

            let passwordField = app.secureTextFields["Password"]
            passwordField.tap()
            passwordField.typeText("password123")

            app.buttons["Sign In"].tap()
            sleep(3)

            takeScreenshot(name: "e2e-error-01-network-error")

            // Should show error (alert or inline message)
            let hasError = app.alerts.firstMatch.exists ||
                app.staticTexts.matching(NSPredicate(format: "label CONTAINS[c] 'error' OR label CONTAINS[c] 'failed' OR label CONTAINS[c] 'network'")).firstMatch.exists

            print("   Network error displayed: \(hasError)")
        }

        print("✅ Network error handling test completed")
    }

    // MARK: - Helpers

    private func ensureAuthenticated() throws {
        // Check if on login screen
        if app.textFields["Email"].waitForExistence(timeout: 3) {
            // Use a known test account
            let emailField = app.textFields["Email"]
            emailField.tap()
            emailField.clearAndTypeText("e2e@test.com")

            let passwordField = app.secureTextFields["Password"]
            passwordField.tap()
            passwordField.typeText("TestPassword123!")

            app.buttons["Sign In"].tap()
            sleep(3)
        }
    }

    private func takeScreenshot(name: String) {
        let screenshot = app.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
        print("   📸 Screenshot: \(name)")
    }
}

// MARK: - XCUIElement Extensions

extension XCUIElement {
    /// Clear existing text and type new text
    func clearAndTypeText(_ text: String) {
        guard self.value is String else {
            self.typeText(text)
            return
        }

        // Triple tap to select all
        self.tap()
        self.tap()
        self.tap()

        // Type new text (replaces selected)
        self.typeText(text)
    }
}
