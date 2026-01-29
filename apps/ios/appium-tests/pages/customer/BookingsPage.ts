import { ids } from '../../utils/ids.js';
import { tap, expectVisible } from '../../utils/helpers.js';

export const BookingsPage = {
	async open() {
		await tap(ids.customer.bookingsTab);
	},
	async openFirstBooking() {
		await tap(ids.customer.bookingItem('first'));
	},
	async reschedule() {
		await tap(ids.customer.bookingReschedule);
	},
	async cancel() {
		await tap(ids.customer.bookingCancel);
		await tap(ids.customer.bookingCancelConfirm);
	},
	async filter() {
		await tap(ids.customer.bookingFilter);
	},
	async expectList() {
		await expectVisible(ids.customer.bookingsList);
	},
};
