import { test, expect } from '@playwright/test';

/**
 * Staging Smoke Tests
 *
 * These tests run against the staging environment to verify
 * critical functionality is working after deployments.
 */

test.describe('Staging Smoke Tests', () => {
  test.describe('Health Checks', () => {
    test('API health endpoint responds', async ({ request }) => {
      const response = await request.get('/health');
      expect(response.ok()).toBeTruthy();

      const data = await response.json();
      expect(data.status).toBe('ok');
    });

    test('customer web is accessible', async ({ page }) => {
      await page.goto('/');
      await expect(page).toHaveTitle(/OFFLEASH|Dog|Pet/i);
    });
  });

  test.describe('Authentication', () => {
    test('login page loads', async ({ page }) => {
      await page.goto('/login');

      await expect(page.locator('input[name="email"]')).toBeVisible();
      await expect(page.locator('input[name="password"]')).toBeVisible();
    });

    test('registration page loads', async ({ page }) => {
      await page.goto('/register');

      // Should show registration form or redirect to login with link
      const hasRegForm = await page.locator('input[name="email"]').isVisible();
      const hasLoginLink = await page.locator('a:has-text("Sign up"), a:has-text("Register")').isVisible();

      expect(hasRegForm || hasLoginLink).toBeTruthy();
    });
  });

  test.describe('Public Pages', () => {
    test('homepage loads', async ({ page }) => {
      await page.goto('/');

      // Should have some content
      const body = page.locator('body');
      await expect(body).not.toBeEmpty();
    });

    test('no console errors on homepage', async ({ page }) => {
      const errors: string[] = [];
      page.on('console', (msg) => {
        if (msg.type() === 'error') {
          errors.push(msg.text());
        }
      });

      await page.goto('/');
      await page.waitForTimeout(2000);

      // Filter out expected errors (like missing favicons)
      const criticalErrors = errors.filter(
        (e) => !e.includes('favicon') && !e.includes('404')
      );

      expect(criticalErrors.length).toBe(0);
    });
  });

  test.describe('API Endpoints', () => {
    test('branding endpoint responds', async ({ request }) => {
      // Branding is often public
      const response = await request.get('/api/branding');
      // May return 200 or 401 depending on configuration
      expect([200, 401]).toContain(response.status());
    });
  });

  test.describe('Performance', () => {
    test('homepage loads within acceptable time', async ({ page }) => {
      const startTime = Date.now();
      await page.goto('/');
      const loadTime = Date.now() - startTime;

      // Should load within 5 seconds
      expect(loadTime).toBeLessThan(5000);
    });

    test('login page loads within acceptable time', async ({ page }) => {
      const startTime = Date.now();
      await page.goto('/login');
      const loadTime = Date.now() - startTime;

      // Should load within 3 seconds
      expect(loadTime).toBeLessThan(3000);
    });
  });

  test.describe('SSL/Security', () => {
    test('uses HTTPS', async ({ page }) => {
      const response = await page.goto('/');
      const url = page.url();

      // In staging, should be HTTPS
      if (process.env.STAGING) {
        expect(url).toMatch(/^https:/);
      }
    });
  });
});
