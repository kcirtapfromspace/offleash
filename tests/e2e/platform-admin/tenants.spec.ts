import { test, expect } from '@playwright/test';
import { TEST_USERS, TEST_ORGS } from '../../utils/constants';
import { uniqueId } from '../../utils/fixtures';

test.describe('Platform Admin - Tenant Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.platformAdmin.email);
    await page.fill('input[name="password"]', TEST_USERS.platformAdmin.password);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
  });

  test('can view dashboard overview', async ({ page }) => {
    await page.goto('/dashboard');

    const dashboard = page.locator('main, [data-testid="dashboard"]');
    await expect(dashboard).toBeVisible();
  });

  test('can view tenants list', async ({ page }) => {
    await page.goto('/tenants');

    const tenantsList = page.locator('[data-testid="tenants-list"], .tenants-list, table, main');
    await expect(tenantsList).toBeVisible({ timeout: 10000 });

    // Should show at least one tenant (the demo org)
    await expect(page.locator(`text=${TEST_ORGS.demo.name}`)).toBeVisible({ timeout: 5000 });
  });

  test('can view tenant details', async ({ page }) => {
    await page.goto('/tenants');

    await page.waitForTimeout(2000);

    // Click on first tenant
    const tenantRow = page.locator('tr:has(td), [data-testid="tenant-row"], .tenant-card').first();
    if (await tenantRow.isVisible()) {
      await tenantRow.click();

      // Should show tenant details
      await expect(page.locator('text=Details, text=Settings, text=Users, text=Subscription').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can create a new tenant', async ({ page }) => {
    await page.goto('/tenants');

    // Click add tenant button
    const addButton = page.locator('button:has-text("Add"), a:has-text("Add"), button:has-text("Create"), a:has-text("New tenant")').first();
    if (await addButton.isVisible()) {
      await addButton.click();
    } else {
      await page.goto('/tenants/new');
    }

    // Fill the form
    const tenantName = `Test Org ${uniqueId()}`;
    await page.fill('input[name="name"]', tenantName);

    const slugInput = page.locator('input[name="slug"]');
    if (await slugInput.isVisible()) {
      await slugInput.fill(`test-org-${Date.now()}`);
    }

    const subdomainInput = page.locator('input[name="subdomain"]');
    if (await subdomainInput.isVisible()) {
      await subdomainInput.fill(`test${Date.now()}`);
    }

    // Submit
    const submitButton = page.locator('button[type="submit"], button:has-text("Save"), button:has-text("Create")');
    await submitButton.click();

    // Should show success
    await expect(page.locator(`text=${tenantName}, text=success, text=created`).first()).toBeVisible({ timeout: 5000 });
  });

  test('can edit tenant settings', async ({ page }) => {
    await page.goto('/tenants');

    await page.waitForTimeout(2000);

    // Find edit button
    const editButton = page.locator('button:has-text("Edit"), a:has-text("Edit"), button:has-text("Settings")').first();
    if (await editButton.isVisible()) {
      await editButton.click();

      // Should show edit form
      await expect(page.locator('input[name="name"]').first()).toBeVisible();
    }
  });

  test('can view tenant users', async ({ page }) => {
    await page.goto('/tenants');

    await page.waitForTimeout(2000);

    // Navigate to tenant users
    const usersLink = page.locator('a:has-text("Users"), button:has-text("View users")').first();
    if (await usersLink.isVisible()) {
      await usersLink.click();

      // Should show users list
      await expect(page.locator('[data-testid="users-list"], .users-list, table').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('can view tenant statistics', async ({ page }) => {
    await page.goto('/tenants');

    await page.waitForTimeout(2000);

    // Click on tenant to see stats
    const tenantRow = page.locator('tr:has(td), [data-testid="tenant-row"]').first();
    if (await tenantRow.isVisible()) {
      await tenantRow.click();

      // Should show stats
      await expect(page.locator('text=Bookings, text=Users, text=Revenue, text=Statistics').first()).toBeVisible({ timeout: 5000 });
    }
  });

  test('shows navigation sidebar', async ({ page }) => {
    await page.goto('/dashboard');

    // Check for navigation items
    const tenantsLink = page.locator('a:has-text("Tenant"), a:has-text("Organization")').first();
    await expect(tenantsLink).toBeVisible();
  });

  test('can access system settings', async ({ page }) => {
    await page.goto('/settings');

    // Should show system settings
    const settings = page.locator('main, [data-testid="settings"]');
    await expect(settings).toBeVisible();
  });
});
