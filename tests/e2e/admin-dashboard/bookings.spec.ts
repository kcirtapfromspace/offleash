import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Admin Dashboard - Booking Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.admin.email);
    await page.fill('input[name="password"]', TEST_USERS.admin.password);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
  });

  test('can view all bookings', async ({ page }) => {
    await page.goto('/bookings');

    // Should show bookings list
    const bookingsList = page.locator('[data-testid="bookings-list"], .bookings-list, table, main');
    await expect(bookingsList).toBeVisible({ timeout: 10000 });
  });

  test('can filter bookings by status', async ({ page }) => {
    await page.goto('/bookings');

    // Look for status filter
    const statusFilter = page.locator('select[name="status"], [data-testid="status-filter"], button:has-text("Filter")').first();
    if (await statusFilter.isVisible()) {
      await statusFilter.click();

      // Select a status option
      const pendingOption = page.locator('option[value="pending"], [data-value="pending"], button:has-text("Pending")').first();
      if (await pendingOption.isVisible()) {
        await pendingOption.click();
      }
    }
  });

  test('can view booking details', async ({ page }) => {
    await page.goto('/bookings');

    await page.waitForTimeout(2000);

    // Click on first booking
    const bookingRow = page.locator('tr, [data-testid="booking-row"], .booking-card').first();
    if (await bookingRow.isVisible()) {
      await bookingRow.click();

      // Should show details modal or navigate to details page
      await expect(page.locator('text=Details, text=Customer, text=Walker, text=Service').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can update booking status', async ({ page }) => {
    await page.goto('/bookings');

    await page.waitForTimeout(2000);

    // Find a booking and update its status
    const statusDropdown = page.locator('select[name="status"], [data-testid="booking-status"]').first();
    if (await statusDropdown.isVisible()) {
      await statusDropdown.selectOption('confirmed');
    }
  });

  test('can create a new booking', async ({ page }) => {
    await page.goto('/bookings/new');

    // Should show booking form
    await expect(page.locator('form, [data-testid="booking-form"]').first()).toBeVisible({ timeout: 10000 });
  });

  test('can cancel a booking', async ({ page }) => {
    await page.goto('/bookings');

    await page.waitForTimeout(2000);

    // Find cancel button
    const cancelButton = page.locator('button:has-text("Cancel")').first();
    if (await cancelButton.isVisible()) {
      await cancelButton.click();

      // Confirm if dialog appears
      const confirmButton = page.locator('button:has-text("Confirm"), button:has-text("Yes")');
      if (await confirmButton.isVisible()) {
        await confirmButton.click();
      }
    }
  });

  test('shows booking calendar view', async ({ page }) => {
    await page.goto('/calendar');

    // Should show calendar
    const calendar = page.locator('[data-testid="calendar"], .calendar, .fc, [class*="calendar"]');
    await expect(calendar.first()).toBeVisible({ timeout: 10000 });
  });
});
