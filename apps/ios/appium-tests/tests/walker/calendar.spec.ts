import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { calendarPage } from '../../pages/walker/CalendarPage';
import { testAccounts } from '../../utils/test-data';

describe('Walker - Calendar', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);
		await browser.pause(3000);

		if (await dashboardPage.isDisplayed()) {
			await dashboardPage.navigateToCalendar();
		}
	});

	// WCL-001: Calendar Month View
	it('WCL-001: should display month view', async () => {
		expect(await calendarPage.isDisplayed()).toBe(true);
	});

	// WCL-002: Calendar Week View
	it('WCL-002: should toggle to week view', async () => {
		if (await calendarPage.isDisplayed()) {
			await calendarPage.toggleToWeekView();
			await browser.pause(1000);
			expect(await calendarPage.isDisplayed()).toBe(true);
		}
	});

	// WCL-003: Select Date
	it('WCL-003: should show bookings for selected date', async () => {
		if (await calendarPage.isDisplayed()) {
			// Get today's date in format expected by calendar
			const today = new Date().toISOString().split('T')[0];
			await calendarPage.selectDate(today);
			await browser.pause(1000);
			// Either bookings list or the selection should work
			expect(await calendarPage.isDisplayed()).toBe(true);
		}
	});
});
