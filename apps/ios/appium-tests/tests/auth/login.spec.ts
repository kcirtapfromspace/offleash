import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { servicesPage } from '../../pages/customer/ServicesPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { testAccounts } from '../../utils/test-data';

describe('Authentication - Login', () => {
	beforeEach(async () => {
		await browser.reloadSession();
	});

	// AUTH-001: Role Selection Customer
	it('AUTH-001: should allow selecting customer role', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		expect(await loginPage.isDisplayed()).toBe(true);
	});

	// AUTH-002: Role Selection Walker
	it('AUTH-002: should allow selecting walker role', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		expect(await loginPage.isDisplayed()).toBe(true);
	});

	// AUTH-003: Email Login Success
	it('AUTH-003: should login successfully with valid credentials', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		expect(await servicesPage.isDisplayed()).toBe(true);
	});

	// AUTH-004: Email Login Invalid
	it('AUTH-004: should show error with invalid credentials', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.invalid.email, testAccounts.invalid.password);
		expect(await loginPage.isErrorDisplayed()).toBe(true);
	});

	// AUTH-005: Password Validation - Skip because password strength indicator doesn't exist in LoginView
	it.skip('AUTH-005: should show password strength indicator', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.enterPassword('weak');
		expect(await loginPage.isPasswordStrengthDisplayed()).toBe(true);
	});

	// AUTH-007: Google OAuth (placeholder - requires manual intervention)
	it.skip('AUTH-007: should initiate Google OAuth flow', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.tapGoogleSignIn();
		// OAuth flow requires external handling
	});

	// AUTH-008: Session Persistence
	it('AUTH-008: should persist session across app restart', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();

		// Restart app
		await browser.terminateApp('com.offleash.ios');
		await browser.pause(1000); // Wait for termination to complete
		await browser.activateApp('com.offleash.ios');
		await browser.pause(3000); // Wait for app to fully launch

		// Should still be logged in
		expect(await servicesPage.isDisplayed()).toBe(true);
	});

	// AUTH-010: Logout - Skip for now due to SwiftUI TabView tab bar interaction issues
	// The Profile tab click doesn't reliably switch tabs in XCUITest
	it.skip('AUTH-010: should logout successfully', async () => {
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();

		// Navigate to profile and logout
		const profilePage = await import('../../pages/customer/ProfilePage');
		await profilePage.profilePage.navigateToProfile();
		await profilePage.profilePage.tapLogout();
		await profilePage.profilePage.confirmLogout();

		expect(await roleSelectionPage.isDisplayed()).toBe(true);
	});
});
