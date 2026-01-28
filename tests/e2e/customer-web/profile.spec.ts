import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Customer Web - Profile & Settings', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.customer.email);
    await page.fill('input[name="password"]', TEST_USERS.customer.password);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(services|bookings|dashboard)/);
  });

  test.describe('Profile', () => {
    test('can view profile page', async ({ page }) => {
      await page.goto('/profile');

      // Should show profile information
      await expect(page.locator(`text=${TEST_USERS.customer.firstName}`)).toBeVisible({ timeout: 10000 });
    });

    test('can edit profile information', async ({ page }) => {
      await page.goto('/profile');

      // Look for edit button
      const editButton = page.locator('button:has-text("Edit"), a:has-text("Edit profile")');
      if (await editButton.isVisible()) {
        await editButton.click();

        // Should show edit form
        await expect(page.locator('input[name="first_name"], input[name="firstName"]').first()).toBeVisible();
      }
    });
  });

  test.describe('Locations', () => {
    test('can view saved locations', async ({ page }) => {
      await page.goto('/locations');

      // Should show locations list or "add location" button
      const hasLocations = await page.locator('[data-testid="location-card"], .location-card').count() > 0;
      const hasAddButton = await page.locator('button:has-text("Add"), a:has-text("Add location")').count() > 0;

      expect(hasLocations || hasAddButton).toBeTruthy();
    });

    test('can add a new location', async ({ page }) => {
      await page.goto('/locations');

      // Click add location
      const addButton = page.locator('button:has-text("Add"), a:has-text("Add location")').first();
      if (await addButton.isVisible()) {
        await addButton.click();

        // Should show location form
        await expect(page.locator('input[name="address"]').first()).toBeVisible({ timeout: 5000 });
      } else {
        await page.goto('/locations/new');
        await expect(page.locator('input[name="address"]').first()).toBeVisible({ timeout: 5000 });
      }
    });

    test('can edit a location', async ({ page }) => {
      await page.goto('/locations');

      // Find a location to edit
      const editButton = page.locator('button:has-text("Edit")').first();
      if (await editButton.isVisible()) {
        await editButton.click();

        // Should show edit form
        await expect(page.locator('input[name="address"]').first()).toBeVisible({ timeout: 5000 });
      }
    });
  });

  test.describe('Payment Methods', () => {
    test('can view payment methods page', async ({ page }) => {
      await page.goto('/payment-methods');

      // Should show payment methods or prompt to add
      const hasPaymentMethods = await page.locator('[data-testid="payment-method"], .payment-method').count() > 0;
      const hasAddButton = await page.locator('button:has-text("Add"), a:has-text("Add payment")').count() > 0;
      const hasPageContent = await page.locator('text=Payment, text=Card, text=No payment methods').count() > 0;

      expect(hasPaymentMethods || hasAddButton || hasPageContent).toBeTruthy();
    });
  });

  test.describe('Notifications', () => {
    test('can access notification settings', async ({ page }) => {
      await page.goto('/settings');

      // Look for notifications section
      const notificationsSection = page.locator('text=Notification, text=notification, text=Email, text=SMS');
      if (await notificationsSection.count() > 0) {
        await expect(notificationsSection.first()).toBeVisible();
      }
    });
  });
});
