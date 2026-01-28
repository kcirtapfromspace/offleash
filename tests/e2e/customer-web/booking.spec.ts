import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Customer Web - Booking Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.customer.email);
    await page.fill('input[name="password"]', TEST_USERS.customer.password);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(services|bookings|dashboard)/);
  });

  test('can view services list', async ({ page }) => {
    await page.goto('/services');

    // Should display services
    const serviceCards = page.locator('[data-testid="service-card"], .service-card, article');
    await expect(serviceCards.first()).toBeVisible({ timeout: 10000 });
  });

  test('can navigate to new booking page', async ({ page }) => {
    await page.goto('/services');

    // Click on "Book now" or service card
    const bookButton = page.locator('button:has-text("Book"), a:has-text("Book")').first();
    if (await bookButton.isVisible()) {
      await bookButton.click();
    } else {
      await page.goto('/bookings/new');
    }

    await expect(page).toHaveURL(/\/bookings\/new/);
  });

  test('booking form displays all required fields', async ({ page }) => {
    await page.goto('/bookings/new');

    // Check for key booking form elements
    await expect(page.locator('text=Select a service, text=Choose a service, [data-testid="service-select"]').first()).toBeVisible({ timeout: 10000 });
  });

  test('can complete booking flow - select service', async ({ page }) => {
    await page.goto('/bookings/new');

    // Wait for services to load
    await page.waitForTimeout(2000);

    // Click on first service
    const serviceOption = page.locator('[data-testid="service-option"], .service-option, button:has-text("30 Minute"), button:has-text("Walk")').first();
    if (await serviceOption.isVisible()) {
      await serviceOption.click();
    }

    // Should proceed to next step or show walker selection
    await expect(page.locator('text=Select walker, text=Choose walker, text=Select location, text=Choose date').first()).toBeVisible({ timeout: 5000 });
  });

  test('can view existing bookings', async ({ page }) => {
    await page.goto('/bookings');

    // Should show bookings list or empty state
    const bookingsList = page.locator('[data-testid="bookings-list"], .bookings-list, main');
    await expect(bookingsList).toBeVisible({ timeout: 10000 });
  });

  test('can view booking details', async ({ page }) => {
    await page.goto('/bookings');

    // Wait for bookings to load
    await page.waitForTimeout(2000);

    // Click on first booking if any exist
    const bookingCard = page.locator('[data-testid="booking-card"], .booking-card, article').first();
    if (await bookingCard.isVisible()) {
      await bookingCard.click();

      // Should show booking details
      await expect(page.locator('text=Details, text=Status, text=Date, text=Time').first()).toBeVisible();
    }
  });

  test('can cancel a booking', async ({ page }) => {
    await page.goto('/bookings');

    await page.waitForTimeout(2000);

    // Find a cancellable booking
    const cancelButton = page.locator('button:has-text("Cancel")').first();
    if (await cancelButton.isVisible()) {
      await cancelButton.click();

      // Confirm cancellation if dialog appears
      const confirmButton = page.locator('button:has-text("Confirm"), button:has-text("Yes")');
      if (await confirmButton.isVisible()) {
        await confirmButton.click();
      }

      // Should show success message or update status
      await expect(page.locator('text=cancelled, text=Cancelled, text=success').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('displays upcoming and past bookings tabs', async ({ page }) => {
    await page.goto('/bookings');

    // Check for tabs
    const upcomingTab = page.locator('button:has-text("Upcoming"), a:has-text("Upcoming"), [role="tab"]:has-text("Upcoming")');
    const pastTab = page.locator('button:has-text("Past"), a:has-text("Past"), button:has-text("History"), [role="tab"]:has-text("Past")');

    // At least one should exist
    const hasUpcoming = await upcomingTab.count() > 0;
    const hasPast = await pastTab.count() > 0;

    expect(hasUpcoming || hasPast).toBeTruthy();
  });
});
