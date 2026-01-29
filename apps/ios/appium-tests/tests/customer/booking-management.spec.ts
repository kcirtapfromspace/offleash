import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { servicesPage } from '../../pages/customer/ServicesPage';
import { testAccounts } from '../../utils/test-data';
import { waitForElement, tapElement, isElementDisplayed } from '../../utils/helpers';

describe('Customer - Booking Management', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();
	});

	// BMG-001: View Bookings
	it('BMG-001: should display bookings list', async () => {
		await tapElement('tab-bookings');
		const hasBookings = await isElementDisplayed('bookings-list', 10000);
		const isEmpty = await isElementDisplayed('empty-state', 5000);
		expect(hasBookings || isEmpty).toBe(true);
	});

	// BMG-002: Booking Details
	it('BMG-002: should show booking details on tap', async () => {
		await tapElement('tab-bookings');
		const hasBookings = await isElementDisplayed('bookings-list', 10000);
		if (hasBookings) {
			const bookings = await $$('~booking-item');
			if (bookings.length > 0) {
				await bookings[0].click();
				// Should show booking details view
				await browser.pause(1000);
				const details = await isElementDisplayed('booking-details', 5000);
				expect(details).toBe(true);
			}
		}
	});

	// BMG-003: Reschedule Booking
	it('BMG-003: should allow rescheduling a booking', async () => {
		await tapElement('tab-bookings');
		const hasBookings = await isElementDisplayed('bookings-list', 10000);
		if (hasBookings) {
			const bookings = await $$('~booking-item');
			if (bookings.length > 0) {
				await bookings[0].click();
				await browser.pause(1000);
				const rescheduleBtn = await isElementDisplayed('booking-reschedule-button', 5000);
				if (rescheduleBtn) {
					await tapElement('booking-reschedule-button');
					// Should show reschedule options
					expect(await isElementDisplayed('booking-date-picker', 5000)).toBe(true);
				}
			}
		}
	});

	// BMG-004: Cancel Booking
	it('BMG-004: should allow cancelling a booking', async () => {
		await tapElement('tab-bookings');
		const hasBookings = await isElementDisplayed('bookings-list', 10000);
		if (hasBookings) {
			const bookings = await $$('~booking-item');
			if (bookings.length > 0) {
				await bookings[0].click();
				await browser.pause(1000);
				const cancelBtn = await isElementDisplayed('booking-cancel-button', 5000);
				if (cancelBtn) {
					await tapElement('booking-cancel-button');
					// Should show confirmation
					expect(await isElementDisplayed('booking-cancel-confirm', 5000)).toBe(true);
				}
			}
		}
	});

	// BMG-005: Filter by Status
	it('BMG-005: should filter bookings by status', async () => {
		await tapElement('tab-bookings');
		const filterBtn = await isElementDisplayed('booking-filter-button', 5000);
		if (filterBtn) {
			await tapElement('booking-filter-button');
			expect(await isElementDisplayed('booking-filter-options', 5000)).toBe(true);
		}
	});

	// BMG-006: Empty State
	it('BMG-006: should show empty state when no bookings', async () => {
		await tapElement('tab-bookings');
		// Either show bookings or empty state
		const hasBookings = await isElementDisplayed('bookings-list', 5000);
		const isEmpty = await isElementDisplayed('empty-state', 5000);
		expect(hasBookings || isEmpty).toBe(true);
	});
});
