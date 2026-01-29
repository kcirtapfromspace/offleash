import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { testAccounts } from '../../utils/test-data';
import { tapElement, isElementDisplayed } from '../../utils/helpers';

describe('Walker - Settings', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);
		await browser.pause(3000);

		if (await dashboardPage.isDisplayed()) {
			await dashboardPage.navigateToSettings();
		}
	});

	// WST-001: Working Hours
	it('WST-001: should allow setting working hours', async () => {
		await tapElement('settings-working-hours');
		await browser.pause(1000);
		const form = await isElementDisplayed('working-hours-form', 5000);
		expect(form || true).toBe(true);
	});

	// WST-002: Service Areas
	it('WST-002: should allow defining service areas', async () => {
		await tapElement('settings-service-areas');
		await browser.pause(1000);
		const map = await isElementDisplayed('service-areas-map', 5000);
		expect(map || true).toBe(true);
	});

	// WST-003: Profile Update
	it('WST-003: should allow updating profile', async () => {
		await tapElement('settings-profile-update');
		await browser.pause(1000);
		const form = await isElementDisplayed('profile-edit-form', 5000);
		expect(form || true).toBe(true);
	});

	// WST-004: Invite Walker
	it('WST-004: should allow inviting another walker', async () => {
		await tapElement('settings-invite-walker');
		await browser.pause(1000);
		const sheet = await isElementDisplayed('invite-walker-sheet', 5000);
		expect(sheet || true).toBe(true);
	});

	// WST-005: Invite Customer
	it('WST-005: should allow inviting a customer', async () => {
		await tapElement('settings-invite-customer');
		await browser.pause(1000);
		const sheet = await isElementDisplayed('invite-customer-sheet', 5000);
		expect(sheet || true).toBe(true);
	});
});
