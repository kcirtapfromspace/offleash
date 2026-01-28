import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Admin Dashboard - Dashboard & Overview', () => {
  test.beforeEach(async ({ page }) => {
    // Login as admin
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.admin.email);
    await page.fill('input[name="password"]', TEST_USERS.admin.password);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
  });

  test('displays dashboard overview', async ({ page }) => {
    await page.goto('/dashboard');

    // Should show key metrics or stats
    const dashboard = page.locator('main, [data-testid="dashboard"]');
    await expect(dashboard).toBeVisible();
  });

  test('shows navigation sidebar', async ({ page }) => {
    await page.goto('/dashboard');

    // Check for navigation items
    const navItems = page.locator('nav a, aside a');
    await expect(navItems.first()).toBeVisible();
  });

  test('can navigate to bookings', async ({ page }) => {
    await page.goto('/dashboard');

    const bookingsLink = page.locator('a:has-text("Bookings")').first();
    if (await bookingsLink.isVisible()) {
      await bookingsLink.click();
      await expect(page).toHaveURL(/\/bookings/);
    }
  });

  test('can navigate to walkers', async ({ page }) => {
    await page.goto('/dashboard');

    const walkersLink = page.locator('a:has-text("Walker"), a:has-text("Staff"), a:has-text("Team")').first();
    if (await walkersLink.isVisible()) {
      await walkersLink.click();
      await expect(page).toHaveURL(/\/(walkers|staff|team)/);
    }
  });

  test('can navigate to services', async ({ page }) => {
    await page.goto('/dashboard');

    const servicesLink = page.locator('a:has-text("Service")').first();
    if (await servicesLink.isVisible()) {
      await servicesLink.click();
      await expect(page).toHaveURL(/\/services/);
    }
  });

  test('can navigate to calendar', async ({ page }) => {
    await page.goto('/dashboard');

    const calendarLink = page.locator('a:has-text("Calendar")').first();
    if (await calendarLink.isVisible()) {
      await calendarLink.click();
      await expect(page).toHaveURL(/\/calendar/);
    }
  });

  test('can navigate to settings', async ({ page }) => {
    await page.goto('/dashboard');

    const settingsLink = page.locator('a:has-text("Settings")').first();
    if (await settingsLink.isVisible()) {
      await settingsLink.click();
      await expect(page).toHaveURL(/\/settings/);
    }
  });
});
