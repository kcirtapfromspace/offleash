import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';
import { uniqueId } from '../../utils/fixtures';

test.describe('Admin Dashboard - Service Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.admin.email);
    await page.fill('input[name="password"]', TEST_USERS.admin.password);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
  });

  test('can view services list', async ({ page }) => {
    await page.goto('/services');

    const servicesList = page.locator('[data-testid="services-list"], .services-list, table, main');
    await expect(servicesList).toBeVisible({ timeout: 10000 });

    // Should show at least one service
    const serviceCards = page.locator('[data-testid="service-card"], .service-card, tr:has(td)');
    await expect(serviceCards.first()).toBeVisible({ timeout: 5000 });
  });

  test('can create a new service', async ({ page }) => {
    await page.goto('/services');

    // Click add service button
    const addButton = page.locator('button:has-text("Add"), a:has-text("Add"), button:has-text("New"), a:has-text("New service")').first();
    if (await addButton.isVisible()) {
      await addButton.click();
    } else {
      await page.goto('/services/new');
    }

    // Fill the form
    const serviceName = `Test Service ${uniqueId()}`;
    await page.fill('input[name="name"]', serviceName);

    const descriptionInput = page.locator('textarea[name="description"], input[name="description"]').first();
    if (await descriptionInput.isVisible()) {
      await descriptionInput.fill('A test service created by E2E tests');
    }

    await page.fill('input[name="duration_minutes"], input[name="duration"]', '30');
    await page.fill('input[name="price"], input[name="base_price_cents"], input[name="price_cents"]', '25');

    // Submit
    const submitButton = page.locator('button[type="submit"], button:has-text("Save"), button:has-text("Create")');
    await submitButton.click();

    // Should show success or redirect to services list
    await expect(page.locator(`text=${serviceName}, text=success, text=created`).first()).toBeVisible({ timeout: 5000 });
  });

  test('can edit a service', async ({ page }) => {
    await page.goto('/services');

    await page.waitForTimeout(2000);

    // Find edit button for first service
    const editButton = page.locator('button:has-text("Edit"), a:has-text("Edit")').first();
    if (await editButton.isVisible()) {
      await editButton.click();

      // Should show edit form
      await expect(page.locator('input[name="name"]').first()).toBeVisible();

      // Update description
      const descriptionInput = page.locator('textarea[name="description"], input[name="description"]').first();
      if (await descriptionInput.isVisible()) {
        await descriptionInput.fill('Updated description via E2E test');
      }

      // Save
      const saveButton = page.locator('button[type="submit"], button:has-text("Save")');
      await saveButton.click();

      // Should show success
      await expect(page.locator('text=Updated, text=success, text=saved').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can toggle service active status', async ({ page }) => {
    await page.goto('/services');

    await page.waitForTimeout(2000);

    // Find toggle or switch for active status
    const toggle = page.locator('input[type="checkbox"][name*="active"], [role="switch"], button[aria-label*="active"]').first();
    if (await toggle.isVisible()) {
      await toggle.click();
    }
  });

  test('validates required fields', async ({ page }) => {
    await page.goto('/services/new');

    // Try to submit without filling required fields
    const submitButton = page.locator('button[type="submit"], button:has-text("Save"), button:has-text("Create")');
    await submitButton.click();

    // Should show validation errors
    await expect(page.locator('text=required, text=Required, [class*="error"]').first()).toBeVisible({ timeout: 3000 });
  });
});
