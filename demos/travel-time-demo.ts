import { chromium, Page } from '@playwright/test';
import * as path from 'path';
import * as fs from 'fs';

const BASE_URL = 'http://localhost:5173';

// Test user credentials - using timestamp to ensure unique email
const TEST_USER = {
  firstName: 'Demo',
  lastName: 'Customer',
  email: `demo.customer.${Date.now()}@test.com`,
  phone: '555-123-4567',
  password: 'TestPassword123!'
};

// Create directories
const screenshotsDir = path.join(__dirname, 'screenshots');
const recordingsDir = path.join(__dirname, 'recordings');
fs.mkdirSync(screenshotsDir, { recursive: true });
fs.mkdirSync(recordingsDir, { recursive: true });

async function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function takeScreenshot(page: Page, name: string) {
  const filePath = path.join(screenshotsDir, `${name}.png`);
  await page.screenshot({ path: filePath, fullPage: true });
  console.log(`  📸 Screenshot: ${name}.png`);
}

async function recordTravelTimeDemo() {
  console.log('🎬 Starting Travel Time Feature Demo...\n');

  const browser = await chromium.launch({
    headless: false,
    slowMo: 300,
  });

  const context = await browser.newContext({
    viewport: { width: 1400, height: 900 },
    recordVideo: {
      dir: recordingsDir,
      size: { width: 1400, height: 900 },
    },
  });

  const page = await context.newPage();

  try {
    // Step 1: Go to homepage
    console.log('1️⃣  Opening customer booking app...');
    await page.goto(BASE_URL, { timeout: 30000, waitUntil: 'networkidle' });
    await delay(1000);
    await takeScreenshot(page, '01-homepage');

    // Check current URL to understand state
    const currentUrl = page.url();
    console.log(`   Current URL: ${currentUrl}`);

    // Step 2: Navigate to booking page or login first
    if (currentUrl.includes('login')) {
      console.log('2️⃣  Registering a new test customer...');

      // Click Register link
      await page.click('a:has-text("Register")');
      await delay(1000);
      await takeScreenshot(page, '02-register-page');

      // Fill registration form
      await page.fill('input[name="firstName"]', TEST_USER.firstName);
      await page.fill('input[name="lastName"]', TEST_USER.lastName);
      await page.fill('input[name="email"]', TEST_USER.email);
      await page.fill('input[name="phone"]', TEST_USER.phone);
      await page.fill('input[name="password"]', TEST_USER.password);
      await page.fill('input[name="confirmPassword"]', TEST_USER.password);
      await takeScreenshot(page, '03-registration-filled');

      console.log(`   Registering as: ${TEST_USER.email}`);
      await page.click('button[type="submit"]');
      await delay(2000);
      await takeScreenshot(page, '04-after-registration');

      // Check if we're logged in now
      const afterRegUrl = page.url();
      console.log(`   After registration URL: ${afterRegUrl}`);
    }

    // Step 3: Add a location first (required for booking)
    console.log('3️⃣  Adding a customer location...');
    await page.goto(`${BASE_URL}/locations`, { timeout: 30000, waitUntil: 'networkidle' });
    await delay(1000);
    await takeScreenshot(page, '05-locations-page');

    // Click "Add Location" button
    await page.click('button:has-text("Add Location")');
    await delay(500);

    // Fill in location form
    await page.fill('input[name="name"]', 'Home');
    await page.fill('input[name="address"]', '1600 Pennsylvania Avenue NW');
    await page.fill('input[name="city"]', 'Denver');
    await page.fill('input[name="state"]', 'CO');
    await page.fill('input[name="zip_code"]', '80202');
    await page.fill('textarea[name="notes"]', 'Front door, ring doorbell');
    await page.check('input[name="is_default"]');
    await takeScreenshot(page, '06-location-form-filled');

    // Submit the form
    await page.click('button:has-text("Save Location")');
    await delay(2000);
    await takeScreenshot(page, '07-location-saved');
    console.log('   ✓ Location added');

    // Step 5: Go to new booking page
    console.log('4️⃣  Navigating to new booking page...');
    await page.goto(`${BASE_URL}/bookings/new`, { timeout: 30000, waitUntil: 'networkidle' });
    await delay(1500);
    await takeScreenshot(page, '08-booking-page');

    // Step 6: Fill in booking form
    console.log('5️⃣  Filling in booking details...');

    // Select service if dropdown exists
    const serviceSelect = page.locator('select[name="service_id"]');
    if (await serviceSelect.isVisible({ timeout: 3000 }).catch(() => false)) {
      const options = await serviceSelect.locator('option').all();
      if (options.length > 1) {
        const value = await options[1].getAttribute('value');
        if (value) {
          await serviceSelect.selectOption(value);
          console.log('   ✓ Selected service');
          await delay(500);
        }
      }
    }

    // Select location if dropdown exists
    const locationSelect = page.locator('select[name="location_id"]');
    if (await locationSelect.isVisible({ timeout: 3000 }).catch(() => false)) {
      const options = await locationSelect.locator('option').all();
      if (options.length > 1) {
        const value = await options[1].getAttribute('value');
        if (value) {
          await locationSelect.selectOption(value);
          console.log('   ✓ Selected location');
          await delay(500);
        }
      }
    }

    await takeScreenshot(page, '09-service-location-selected');

    // Select date (tomorrow)
    console.log('6️⃣  Selecting date...');
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    const dateStr = tomorrow.toISOString().split('T')[0];

    const dateInput = page.locator('input[type="date"]');
    if (await dateInput.isVisible({ timeout: 3000 }).catch(() => false)) {
      await dateInput.fill(dateStr);
      console.log(`   ✓ Selected date: ${dateStr}`);
      await delay(2000); // Wait for slots to load
    }

    await takeScreenshot(page, '10-date-selected');

    // Step 7: Show time slots with travel time
    console.log('7️⃣  Showing time slots with travel time indicators...');
    await delay(1500);
    await takeScreenshot(page, '11-time-slots-with-travel');

    // Look for time slot buttons
    const timeSlots = page.locator('button:has-text("AM"), button:has-text("PM")');
    const slotsCount = await timeSlots.count();
    console.log(`   Found ${slotsCount} time slots`);

    if (slotsCount > 0) {
      // Click on a time slot
      console.log('8️⃣  Selecting a time slot...');
      await timeSlots.first().click();
      await delay(1000);
      await takeScreenshot(page, '12-slot-selected');

      // Scroll to see the booking summary
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
      await delay(500);
      await takeScreenshot(page, '13-booking-summary');
    }

    // Look for travel time indicators
    console.log('9️⃣  Checking for travel time indicators...');
    const travelIndicator = page.locator('text=/\\d+ min away/i');
    if (await travelIndicator.isVisible({ timeout: 2000 }).catch(() => false)) {
      console.log('   ✓ Found travel time indicators!');
      await travelIndicator.first().scrollIntoViewIfNeeded();
      await takeScreenshot(page, '14-travel-time-indicator');
    }

    // Look for tight schedule warnings (yellow elements)
    const warnings = page.locator('[class*="yellow"]');
    const warningCount = await warnings.count();
    if (warningCount > 0) {
      console.log(`   ⚠️  Found ${warningCount} tight schedule warnings`);
      await takeScreenshot(page, '15-schedule-warnings');
    }

    // Final full page screenshot
    await page.evaluate(() => window.scrollTo(0, 0));
    await delay(500);
    await takeScreenshot(page, '16-final-overview');

    console.log('\n✅ Demo recording complete!');

  } catch (error) {
    console.error('❌ Demo error:', error);
    try {
      await takeScreenshot(page, 'error-state');
    } catch (e) {
      // Ignore screenshot error
    }
  } finally {
    console.log('\n📁 Saving files...');
    await page.close();
    await context.close();
    await browser.close();

    console.log(`\n📸 Screenshots: ${screenshotsDir}`);
    console.log(`🎥 Video: ${recordingsDir}`);
  }
}

// Run
recordTravelTimeDemo().catch(console.error);
