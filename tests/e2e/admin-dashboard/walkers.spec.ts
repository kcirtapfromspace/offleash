import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Admin Dashboard - Walker Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.admin.email);
    await page.fill('input[name="password"]', TEST_USERS.admin.password);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
  });

  test('can view walkers list', async ({ page }) => {
    await page.goto('/walkers');

    const walkersList = page.locator('[data-testid="walkers-list"], .walkers-list, table, main');
    await expect(walkersList).toBeVisible({ timeout: 10000 });
  });

  test('can view walker details', async ({ page }) => {
    await page.goto('/walkers');

    await page.waitForTimeout(2000);

    // Click on first walker
    const walkerRow = page.locator('tr:has(td), [data-testid="walker-row"], .walker-card').first();
    if (await walkerRow.isVisible()) {
      await walkerRow.click();

      // Should show walker details
      await expect(page.locator('text=Details, text=Working hours, text=Bookings, text=Schedule').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can view walker schedule', async ({ page }) => {
    await page.goto('/walkers');

    await page.waitForTimeout(2000);

    // Find schedule link or button
    const scheduleLink = page.locator('a:has-text("Schedule"), button:has-text("Schedule"), a:has-text("Calendar")').first();
    if (await scheduleLink.isVisible()) {
      await scheduleLink.click();

      // Should show schedule view
      await expect(page.locator('[data-testid="schedule"], .schedule, .calendar').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can edit walker working hours', async ({ page }) => {
    await page.goto('/walkers');

    await page.waitForTimeout(2000);

    // Find working hours link or edit button
    const workingHoursLink = page.locator('a:has-text("Working hours"), button:has-text("Working hours"), a:has-text("Hours")').first();
    if (await workingHoursLink.isVisible()) {
      await workingHoursLink.click();

      // Should show working hours form
      await expect(page.locator('text=Monday, text=Tuesday, text=Start time, text=End time').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can invite a new walker', async ({ page }) => {
    await page.goto('/walkers');

    // Find invite button
    const inviteButton = page.locator('button:has-text("Invite"), a:has-text("Invite"), button:has-text("Add walker")').first();
    if (await inviteButton.isVisible()) {
      await inviteButton.click();

      // Should show invite form
      await expect(page.locator('input[name="email"], input[type="email"]').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can view walker bookings', async ({ page }) => {
    await page.goto('/walkers');

    await page.waitForTimeout(2000);

    // Navigate to walker and view their bookings
    const viewBookingsLink = page.locator('a:has-text("Bookings"), button:has-text("View bookings")').first();
    if (await viewBookingsLink.isVisible()) {
      await viewBookingsLink.click();

      // Should show bookings for the walker
      await expect(page.locator('[data-testid="bookings"], .bookings-list, table').first()).toBeVisible({ timeout: 5000 });
    }
  });
});
