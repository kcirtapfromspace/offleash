import { ids } from '../../utils/ids.js';
import { tap, expectVisible } from '../../utils/helpers.js';

export const OrgSwitchPage = {
	async openSwitcher() {
		await tap(ids.multiTenant.orgSwitcher);
	},
	async switchTo(orgName: string) {
		await tap(ids.multiTenant.orgItem(orgName));
	},
	async expectBranding() {
		await expectVisible(ids.multiTenant.brandingLogo);
	},
	async expectRoleBadge() {
		await expectVisible(ids.multiTenant.roleBadge);
	},
};
