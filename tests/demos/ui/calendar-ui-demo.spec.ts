import { test, expect } from '@playwright/test';

/**
 * Calendar & Service Area UI Demo
 *
 * This test suite demonstrates:
 * 1. Calendar view functionality
 * 2. Calendar blocking UI
 * 3. Service area management with polygon drawing
 * 4. Working hours configuration
 *
 * Run with (requires admin dashboard running on 5174):
 *   npx playwright test tests/demos/calendar-ui-demo.spec.ts --project=demos-ui --headed
 *
 * Generate video:
 *   npx playwright test tests/demos/calendar-ui-demo.spec.ts --project=demos-ui --video=on
 */

test.describe('Calendar UI Demo', () => {
  test.describe.configure({ mode: 'serial' });

  test('1. Calendar View - Display walker schedule', async ({ page }) => {
    // Navigate to calendar - storage state should handle auth via cookies
    await page.goto('/calendar');
    await page.waitForLoadState('networkidle');

    // Verify we're on calendar page (not redirected to login)
    await expect(page).toHaveURL(/\/calendar/);

    // Take screenshot of week view
    await expect(page.locator('h1')).toContainText('Calendar');
    await page.screenshot({ path: 'test-results/demo-calendar-week-view.png', fullPage: true });

    // Switch to day view
    await page.click('button:has-text("Day")');
    await page.waitForTimeout(500);
    await page.screenshot({ path: 'test-results/demo-calendar-day-view.png', fullPage: true });

    // Switch to month view
    await page.click('button:has-text("Month")');
    await page.waitForTimeout(500);
    await page.screenshot({ path: 'test-results/demo-calendar-month-view.png', fullPage: true });

    // Switch to agenda view
    await page.click('button:has-text("Agenda")');
    await page.waitForTimeout(500);
    await page.screenshot({ path: 'test-results/demo-calendar-agenda-view.png', fullPage: true });
  });

  test('2. Calendar Blocking - Create time blocks', async ({ page }) => {
    await page.goto('/calendar');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/calendar/);

    // Click "Block Time" button
    await page.click('button:has-text("Block Time")');
    await expect(page.locator('h3:has-text("Block Time")')).toBeVisible();
    await page.screenshot({ path: 'test-results/demo-block-modal.png' });

    // Fill in block details
    await page.fill('input[name="title"]', 'Lunch Break - Demo');

    // Set start time to tomorrow at noon
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    tomorrow.setHours(12, 0, 0, 0);
    const startTime = tomorrow.toISOString().slice(0, 16);

    // Set end time to tomorrow at 1pm
    const endTime = new Date(tomorrow);
    endTime.setHours(13, 0, 0, 0);
    const endTimeStr = endTime.toISOString().slice(0, 16);

    await page.fill('input[name="start_time"]', startTime);
    await page.fill('input[name="end_time"]', endTimeStr);

    // Ensure blocking checkbox is checked
    await page.check('input[name="is_blocking"]');
    await page.screenshot({ path: 'test-results/demo-block-filled.png' });

    // Close modal without saving (demo only)
    await page.click('button:has-text("Cancel")');
  });

  test('3. Recurring Blocks - Create weekly lunch block', async ({ page }) => {
    await page.goto('/calendar');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/calendar/);

    // Click "Recurring Block" button
    await page.click('button:has-text("Recurring Block")');
    await expect(page.locator('h3:has-text("Create Recurring Block")')).toBeVisible();
    await page.screenshot({ path: 'test-results/demo-recurring-modal.png' });

    // Fill in recurring block details within the modal
    const modal = page.locator('[role="dialog"]');
    await modal.locator('input[name="title"]').fill('Daily Lunch Break');
    await modal.locator('input[name="start_time"]').fill('12:00');
    await modal.locator('input[name="end_time"]').fill('13:00');

    // Select weekdays (Mon-Fri) - use exact text match within modal to avoid "Month" button
    for (const day of ['Mon', 'Tue', 'Wed', 'Thu', 'Fri']) {
      await modal.locator(`button:text-is("${day}")`).click();
    }

    await page.screenshot({ path: 'test-results/demo-recurring-filled.png' });

    // Close modal without saving (demo only)
    await modal.locator('button:has-text("Cancel")').click();
  });

  test('4. Service Areas - View and manage walker zones', async ({ page }) => {
    // Navigate to walkers list
    await page.goto('/walkers');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/walkers/);
    await page.screenshot({ path: 'test-results/demo-walkers-list.png', fullPage: true });

    // Click on first walker (if exists)
    const walkerRow = page.locator('table tbody tr').first();
    if ((await walkerRow.count()) > 0) {
      await walkerRow.click();
      await page.waitForLoadState('networkidle');

      // Navigate to Service Areas tab
      await page.click('button:has-text("Service Areas")');
      await page.waitForTimeout(500);
      await page.screenshot({ path: 'test-results/demo-service-areas-tab.png', fullPage: true });

      // Click "Add Area" button
      await page.click('button:has-text("Add Area")');
      await expect(page.locator('h2:has-text("Create Service Area")')).toBeVisible();
      await page.screenshot({ path: 'test-results/demo-service-area-modal.png' });

      // Fill in area details
      await page.fill('input[name="name"]', 'Downtown Denver - Demo');

      // Enable drawing mode
      await page.click('button:has-text("Draw")');

      // Simulate drawing polygon points on the map
      const mapArea = page.locator('.cursor-crosshair');
      if ((await mapArea.count()) > 0) {
        const box = await mapArea.boundingBox();
        if (box) {
          // Click 4 points to create a square
          await page.mouse.click(box.x + box.width * 0.3, box.y + box.height * 0.3);
          await page.mouse.click(box.x + box.width * 0.7, box.y + box.height * 0.3);
          await page.mouse.click(box.x + box.width * 0.7, box.y + box.height * 0.7);
          await page.mouse.click(box.x + box.width * 0.3, box.y + box.height * 0.7);
        }
      }

      await page.screenshot({ path: 'test-results/demo-service-area-polygon.png' });

      // Close modal without saving
      await page.click('button:has-text("Cancel")');
    } else {
      console.log('No walkers found - skipping service areas demo');
    }
  });

  test('5. Working Hours - Configure walker schedule', async ({ page }) => {
    // Navigate to walkers list and select first walker
    await page.goto('/walkers');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/walkers/);

    const walkerRow = page.locator('table tbody tr').first();
    if ((await walkerRow.count()) > 0) {
      await walkerRow.click();
      await page.waitForLoadState('networkidle');

      // Navigate to Schedule tab
      await page.click('button:has-text("Schedule")');
      await page.waitForTimeout(500);
      await page.screenshot({ path: 'test-results/demo-working-hours.png', fullPage: true });
    } else {
      console.log('No walkers found - skipping working hours demo');
    }
  });
});
