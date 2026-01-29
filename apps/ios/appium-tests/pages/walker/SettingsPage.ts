import { ids } from '../../utils/ids.js';
import { tap, expectVisible } from '../../utils/helpers.js';

export const SettingsPage = {
	async open() {
		await tap(ids.walker.settingsTab);
	},
	async setWorkingHours() {
		await tap(ids.walker.workingHours);
		await tap('working-hours-save');
	},
	async setServiceAreas() {
		await tap(ids.walker.serviceAreas);
		await tap('service-areas-save');
	},
	async updateProfile() {
		await tap(ids.walker.profileUpdate);
		await tap('profile-update-save');
	},
	async inviteWalker() {
		await tap(ids.walker.inviteWalker);
	},
	async inviteCustomer() {
		await tap(ids.walker.inviteCustomer);
	},
	async expectVisible() {
		await expectVisible(ids.walker.settingsTab);
	},
};
