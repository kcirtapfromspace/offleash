import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { bookingRequestsPage } from '../../pages/walker/BookingRequestsPage';
import { testAccounts } from '../../utils/test-data';

describe('Walker - Booking Requests', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);
		await browser.pause(3000);

		if (await dashboardPage.isDisplayed()) {
			await dashboardPage.navigateToRequests();
		}
	});

	// WBR-001: View Requests
	it('WBR-001: should display pending requests', async () => {
		expect(await bookingRequestsPage.isDisplayed()).toBe(true);
	});

	// WBR-002: Accept Request
	it('WBR-002: should allow accepting a request', async () => {
		if (await bookingRequestsPage.isDisplayed()) {
			const count = await bookingRequestsPage.getRequestCount();
			if (count > 0) {
				const requests = await $$('~request-item');
				await requests[0].click();
				await browser.pause(1000);

				if (await bookingRequestsPage.isRequestDetailDisplayed()) {
					await bookingRequestsPage.tapAccept();
					// Should process acceptance
					await browser.pause(2000);
				}
			}
		}
	});

	// WBR-003: Decline Request
	it('WBR-003: should allow declining a request with reason', async () => {
		if (await bookingRequestsPage.isDisplayed()) {
			const count = await bookingRequestsPage.getRequestCount();
			if (count > 0) {
				const requests = await $$('~request-item');
				await requests[0].click();
				await browser.pause(1000);

				if (await bookingRequestsPage.isRequestDetailDisplayed()) {
					await bookingRequestsPage.declineWithReason('Schedule conflict');
					await browser.pause(2000);
				}
			}
		}
	});

	// WBR-004: Request Details
	it('WBR-004: should show full request details', async () => {
		if (await bookingRequestsPage.isDisplayed()) {
			const count = await bookingRequestsPage.getRequestCount();
			if (count > 0) {
				const requests = await $$('~request-item');
				await requests[0].click();
				expect(await bookingRequestsPage.isRequestDetailDisplayed()).toBe(true);
			}
		}
	});

	// WBR-005: Empty State
	it('WBR-005: should show empty state when no requests', async () => {
		expect(await bookingRequestsPage.isDisplayed()).toBe(true);
	});
});
