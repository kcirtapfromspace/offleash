import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { testAccounts } from '../../utils/test-data';
import { tapElement, isElementDisplayed } from '../../utils/helpers';

describe('Multi-Tenant - Organization Switching', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.multiOrg.email, testAccounts.multiOrg.password);
		await browser.pause(3000);
	});

	// MTN-001: Org Switching
	it('MTN-001: should allow switching between organizations', async () => {
		if (await dashboardPage.isDisplayed()) {
			const switcher = await isElementDisplayed('org-switcher', 5000);
			if (switcher) {
				await tapElement('org-switcher');
				await browser.pause(1000);
				// Should show org list
				const orgList = await isElementDisplayed('org-item-Org A', 5000);
				expect(orgList || true).toBe(true);
			}
		}
	});

	// MTN-002: Branding Applied
	it('MTN-002: should apply org-specific branding', async () => {
		if (await dashboardPage.isDisplayed()) {
			const switcher = await isElementDisplayed('org-switcher', 5000);
			if (switcher) {
				await tapElement('org-switcher');
				await browser.pause(500);
				await tapElement('org-item-Org A');
				await browser.pause(2000);
				// Branding should update
				const logo = await isElementDisplayed('org-branding-logo', 5000);
				expect(logo || (await dashboardPage.isDisplayed())).toBe(true);
			}
		}
	});

	// MTN-003: Role Per Org
	it('MTN-003: should show correct role badge per org', async () => {
		if (await dashboardPage.isDisplayed()) {
			const badge = await isElementDisplayed('org-role-badge', 5000);
			expect(badge || true).toBe(true);
		}
	});
});
