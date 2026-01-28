import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Admin Dashboard - Authentication', () => {
  test.beforeEach(async ({ page }) => {
    await page.context().clearCookies();
  });

  test('displays login page', async ({ page }) => {
    await page.goto('/login');

    await expect(page.locator('input[name="email"]')).toBeVisible();
    await expect(page.locator('input[name="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('admin can login', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.admin.email);
    await page.fill('input[name="password"]', TEST_USERS.admin.password);
    await page.click('button[type="submit"]');

    // Should redirect to dashboard
    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('owner can login', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.owner.email);
    await page.fill('input[name="password"]', TEST_USERS.owner.password);
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('walker can login', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.walker.email);
    await page.fill('input[name="password"]', TEST_USERS.walker.password);
    await page.click('button[type="submit"]');

    // Walker should be redirected to dashboard or calendar
    await expect(page).toHaveURL(/\/(dashboard|calendar)/);
  });

  test('shows error with invalid credentials', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.admin.email);
    await page.fill('input[name="password"]', 'WrongPassword!');
    await page.click('button[type="submit"]');

    await expect(page.locator('text=Invalid|incorrect|failed')).toBeVisible({ timeout: 5000 });
  });

  test('can logout', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.admin.email);
    await page.fill('input[name="password"]', TEST_USERS.admin.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Logout
    const logoutButton = page.locator('button:has-text("Logout"), a:has-text("Logout")');
    if (await logoutButton.count() > 0) {
      await logoutButton.first().click();
      await expect(page).toHaveURL(/\/login/);
    } else {
      await page.goto('/logout');
      await expect(page).toHaveURL(/\/login/);
    }
  });
});
