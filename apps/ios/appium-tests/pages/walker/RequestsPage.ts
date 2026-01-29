import { ids } from '../../utils/ids.js';
import { tap, typeText, expectVisible } from '../../utils/helpers.js';

export const RequestsPage = {
	async open() {
		await tap(ids.walker.requestsTab);
	},
	async openFirstRequest() {
		await tap(ids.walker.requestItem('first'));
	},
	async accept() {
		await tap(ids.walker.acceptBooking);
	},
	async decline(reason: string) {
		await tap(ids.walker.declineBooking);
		await typeText(ids.walker.declineReason, reason);
		await tap('decline-submit');
	},
	async expectList() {
		await expectVisible(ids.walker.requestsList);
	},
};
