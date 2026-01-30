import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { servicesPage } from '../../pages/customer/ServicesPage';
import { profilePage } from '../../pages/customer/ProfilePage';
import { testAccounts } from '../../utils/test-data';

describe('Customer - Profile', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();
		await profilePage.navigateToProfile();
	});

	// PRF-001: View Profile
	it('PRF-001: should display profile information', async () => {
		expect(await profilePage.isDisplayed()).toBe(true);
	});

	// PRF-002: Edit Profile
	it('PRF-002: should allow editing profile', async () => {
		await profilePage.tapEdit();
		// Edit form should be active
		const saveBtn = await $('~profile-save-button');
		expect(await saveBtn.isDisplayed()).toBe(true);
	});
});
