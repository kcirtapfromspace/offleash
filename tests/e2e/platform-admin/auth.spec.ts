import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Platform Admin - Authentication', () => {
  test.beforeEach(async ({ page }) => {
    await page.context().clearCookies();
  });

  test('displays login page', async ({ page }) => {
    await page.goto('/login');

    await expect(page.locator('input[name="email"]')).toBeVisible();
    await expect(page.locator('input[name="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('platform admin can login', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.platformAdmin.email);
    await page.fill('input[name="password"]', TEST_USERS.platformAdmin.password);
    await page.click('button[type="submit"]');

    // Should redirect to dashboard
    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('regular user cannot access platform admin', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.customer.email);
    await page.fill('input[name="password"]', TEST_USERS.customer.password);
    await page.click('button[type="submit"]');

    // Should show error or stay on login
    await expect(page.locator('text=error, text=unauthorized, text=Invalid').first()).toBeVisible({ timeout: 5000 });
  });

  test('shows error with invalid credentials', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.platformAdmin.email);
    await page.fill('input[name="password"]', 'WrongPassword!');
    await page.click('button[type="submit"]');

    await expect(page.locator('text=Invalid|incorrect|failed')).toBeVisible({ timeout: 5000 });
  });

  test('can logout', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.platformAdmin.email);
    await page.fill('input[name="password"]', TEST_USERS.platformAdmin.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Logout
    const logoutButton = page.locator('button:has-text("Logout"), a:has-text("Logout")');
    if (await logoutButton.count() > 0) {
      await logoutButton.first().click();
    } else {
      await page.goto('/logout');
    }

    await expect(page).toHaveURL(/\/login/);
  });
});
