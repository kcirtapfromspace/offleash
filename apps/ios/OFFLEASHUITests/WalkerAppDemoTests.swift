import XCTest

/// Automated demo tests for the OFFLEASH Walker iOS App
/// Run with: xcodebuild test -scheme OFFLEASH -destination 'platform=iOS Simulator,name=iPhone 15 Pro' -only-testing:OFFLEASHUITests/WalkerAppDemoTests
final class WalkerAppDemoTests: XCTestCase {

    var app: XCUIApplication!
    let screenshotsDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        .appendingPathComponent("DemoScreenshots")

    override func setUpWithError() throws {
        continueAfterFailure = false
        app = XCUIApplication()
        app.launchArguments = ["--uitesting"]

        // Create screenshots directory
        try? FileManager.default.createDirectory(at: screenshotsDir, withIntermediateDirectories: true)

        app.launch()
    }

    override func tearDownWithError() throws {
        // Take final screenshot
        takeScreenshot(name: "99-final-state")
    }

    // MARK: - Demo Flow

    func testWalkerAppDemo() throws {
        print("🎬 Starting Walker App Demo...")

        // Step 1: Login Screen
        print("1️⃣ Login Screen")
        takeScreenshot(name: "01-login-screen")
        sleep(1)

        // Check if we're on login screen
        if app.textFields["Email"].waitForExistence(timeout: 5) {
            // Fill login form
            let emailField = app.textFields["Email"]
            emailField.tap()
            emailField.typeText("alex@demo.com")

            let passwordField = app.secureTextFields["Password"]
            passwordField.tap()
            passwordField.typeText("password123")

            takeScreenshot(name: "02-login-filled")

            // Tap login button
            app.buttons["Sign In"].tap()
            sleep(2)
            takeScreenshot(name: "03-after-login")
        }

        // Step 2: Check for Today's Schedule
        print("2️⃣ Today's Schedule")
        if app.staticTexts["Today's Schedule"].waitForExistence(timeout: 5) {
            takeScreenshot(name: "04-todays-schedule")

            // Look for booking cards
            let bookingCards = app.otherElements.matching(identifier: "BookingCard")
            if bookingCards.count > 0 {
                print("   Found \(bookingCards.count) bookings")
                takeScreenshot(name: "05-booking-cards")
            }
        }

        // Step 3: View a Booking Detail
        print("3️⃣ Booking Detail")
        let firstBooking = app.otherElements.matching(identifier: "BookingCard").firstMatch
        if firstBooking.waitForExistence(timeout: 3) {
            firstBooking.tap()
            sleep(1)
            takeScreenshot(name: "06-booking-detail")

            // Look for navigation button
            if app.buttons["Navigate"].exists {
                takeScreenshot(name: "07-navigate-button")
            }

            // Go back
            app.navigationBars.buttons.firstMatch.tap()
            sleep(1)
        }

        // Step 4: Check Walker Status Toggle
        print("4️⃣ Walker Status")
        if app.switches["On Duty"].waitForExistence(timeout: 3) {
            takeScreenshot(name: "08-duty-status")

            // Toggle duty status
            app.switches["On Duty"].tap()
            sleep(1)
            takeScreenshot(name: "09-duty-toggled")

            // Toggle back
            app.switches["On Duty"].tap()
            sleep(1)
        }

        // Step 5: Location Services
        print("5️⃣ Location")
        // Check if location indicator exists
        if app.images["location.fill"].exists || app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'Location'")).count > 0 {
            takeScreenshot(name: "10-location-status")
        }

        // Step 6: Profile/Settings
        print("6️⃣ Profile")
        let profileTab = app.tabBars.buttons["Profile"]
        if profileTab.exists {
            profileTab.tap()
            sleep(1)
            takeScreenshot(name: "11-profile-screen")
        }

        print("✅ Demo complete!")
        print("📸 Screenshots saved to: \(screenshotsDir.path)")
    }

    // MARK: - Registration Demo

    func testRegistrationFlow() throws {
        print("🎬 Starting Registration Demo...")

        // Check if we need to navigate to register
        if app.buttons["Create Account"].waitForExistence(timeout: 3) {
            app.buttons["Create Account"].tap()
            sleep(1)
        } else if app.buttons["Register"].waitForExistence(timeout: 3) {
            app.buttons["Register"].tap()
            sleep(1)
        }

        takeScreenshot(name: "reg-01-register-screen")

        // Fill registration form
        if app.textFields["First Name"].waitForExistence(timeout: 3) {
            app.textFields["First Name"].tap()
            app.textFields["First Name"].typeText("Demo")

            app.textFields["Last Name"].tap()
            app.textFields["Last Name"].typeText("Walker")

            app.textFields["Email"].tap()
            app.textFields["Email"].typeText("demo.walker.\(Int(Date().timeIntervalSince1970))@test.com")

            app.textFields["Phone"].tap()
            app.textFields["Phone"].typeText("555-123-4567")

            takeScreenshot(name: "reg-02-form-filled")

            app.secureTextFields["Password"].tap()
            app.secureTextFields["Password"].typeText("Password123!")

            if app.secureTextFields["Confirm Password"].exists {
                app.secureTextFields["Confirm Password"].tap()
                app.secureTextFields["Confirm Password"].typeText("Password123!")
            }

            takeScreenshot(name: "reg-03-password-filled")
        }

        print("✅ Registration demo complete!")
    }

    // MARK: - Travel Time Demo

    func testTravelTimeIndicators() throws {
        print("🎬 Starting Travel Time Demo...")

        // Login first
        try loginAsWalker()

        // Look for travel time indicators
        if app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'min away'")).firstMatch.waitForExistence(timeout: 5) {
            takeScreenshot(name: "travel-01-time-indicators")

            // Find tight schedule warnings (yellow elements)
            let tightWarnings = app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'tight' OR label CONTAINS 'Tight'"))
            if tightWarnings.count > 0 {
                print("   ⚠️ Found \(tightWarnings.count) tight schedule warnings")
                takeScreenshot(name: "travel-02-tight-warnings")
            }
        }

        // Look for next booking with travel info
        let nextBookingSection = app.otherElements["NextBookingCard"]
        if nextBookingSection.exists {
            takeScreenshot(name: "travel-03-next-booking")

            // Check for travel time display
            if nextBookingSection.staticTexts.matching(NSPredicate(format: "label MATCHES '.*\\\\d+ min.*'")).count > 0 {
                takeScreenshot(name: "travel-04-travel-time-display")
            }
        }

        print("✅ Travel time demo complete!")
    }

    // MARK: - Helpers

    private func loginAsWalker() throws {
        if app.textFields["Email"].waitForExistence(timeout: 3) {
            app.textFields["Email"].tap()
            app.textFields["Email"].typeText("alex@demo.com")

            app.secureTextFields["Password"].tap()
            app.secureTextFields["Password"].typeText("password123")

            app.buttons["Sign In"].tap()
            sleep(2)
        }
    }

    private func takeScreenshot(name: String) {
        let screenshot = app.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)

        // Also save to file
        let fileURL = screenshotsDir.appendingPathComponent("\(name).png")
        do {
            try screenshot.pngRepresentation.write(to: fileURL)
            print("   📸 Screenshot: \(name).png")
        } catch {
            print("   ⚠️ Failed to save screenshot: \(error)")
        }
    }
}
