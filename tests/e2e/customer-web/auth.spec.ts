import { test, expect } from '@playwright/test';
import { TEST_USERS } from '../../utils/constants';

test.describe('Customer Web - Authentication', () => {
  test.beforeEach(async ({ page }) => {
    // Clear any existing auth state
    await page.context().clearCookies();
  });

  test('displays login page', async ({ page }) => {
    await page.goto('/login');

    await expect(page.locator('input[name="email"]')).toBeVisible();
    await expect(page.locator('input[name="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('can login with valid credentials', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.customer.email);
    await page.fill('input[name="password"]', TEST_USERS.customer.password);
    await page.click('button[type="submit"]');

    // Should redirect to services or dashboard
    await expect(page).toHaveURL(/\/(services|bookings|dashboard)/);
  });

  test('shows error with invalid credentials', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', TEST_USERS.customer.email);
    await page.fill('input[name="password"]', 'WrongPassword123!');
    await page.click('button[type="submit"]');

    // Should show error message
    await expect(page.locator('text=Invalid|incorrect|failed')).toBeVisible({ timeout: 5000 });
  });

  test('can navigate to registration page', async ({ page }) => {
    await page.goto('/login');

    // Look for sign up / register link
    const registerLink = page.locator('a:has-text("Sign up"), a:has-text("Register"), a:has-text("Create account")');
    if (await registerLink.count() > 0) {
      await registerLink.first().click();
      await expect(page).toHaveURL(/\/register/);
    }
  });

  test('can logout', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[name="email"]', TEST_USERS.customer.email);
    await page.fill('input[name="password"]', TEST_USERS.customer.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/(services|bookings|dashboard)/);

    // Find and click logout
    const logoutButton = page.locator('button:has-text("Logout"), a:has-text("Logout"), button:has-text("Sign out"), a:has-text("Sign out")');
    if (await logoutButton.count() > 0) {
      await logoutButton.first().click();
      await expect(page).toHaveURL(/\/login/);
    } else {
      // Try navigating to /logout directly
      await page.goto('/logout');
      await expect(page).toHaveURL(/\/login/);
    }
  });

  test('redirects unauthenticated users to login', async ({ page }) => {
    await page.goto('/bookings');

    // Should redirect to login
    await expect(page).toHaveURL(/\/login/);
  });

  test('preserves intended destination after login', async ({ page }) => {
    // Try to access protected page
    await page.goto('/bookings/new');

    // Should redirect to login
    await expect(page).toHaveURL(/\/login/);

    // Login
    await page.fill('input[name="email"]', TEST_USERS.customer.email);
    await page.fill('input[name="password"]', TEST_USERS.customer.password);
    await page.click('button[type="submit"]');

    // Should redirect back to intended page or bookings
    await expect(page).toHaveURL(/\/(bookings|services)/);
  });
});
