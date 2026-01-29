import { ids } from '../../utils/ids.js';
import { tap, expectAnyVisible, expectVisible } from '../../utils/helpers.js';

export const PaymentsPage = {
	async open() {
		await tap(ids.customer.tabBarProfile);
		await expectAnyVisible([
			ids.customer.profileHeader,
			ids.customer.paymentsTab,
			ids.auth.logoutButton,
		]);
		await tap(ids.customer.paymentsTab);
	},
	async addCard() {
		await tap(ids.customer.paymentAdd);
	},
	async removeCard() {
		await tap(ids.customer.paymentRemove);
	},
	async viewHistory() {
		await tap(ids.customer.paymentHistory);
	},
	async viewSubscriptions() {
		await tap(ids.customer.subscriptions);
	},
	async expectMethods() {
		await expectVisible(ids.customer.paymentMethodsList);
	},
};
