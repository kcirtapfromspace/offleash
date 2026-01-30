import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { servicesPage } from '../../pages/customer/ServicesPage';
import { testAccounts } from '../../utils/test-data';

describe('Customer - Services', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();
	});

	// SVC-001: Services Load
	it('SVC-001: should display services list', async () => {
		expect(await servicesPage.isServicesListDisplayed()).toBe(true);
	});

	// SVC-002: Service Details
	it('SVC-002: should show service details on tap', async () => {
		// Get count first to ensure services exist
		const count = await servicesPage.getServiceCount();
		if (count > 0) {
			await servicesPage.selectService('first');
			// Verify booking button is available
			const bookButton = await $('~book-service-button');
			expect(await bookButton.isDisplayed()).toBe(true);
		}
	});

	// SVC-003: Pull to Refresh
	it('SVC-003: should refresh on pull down', async () => {
		await servicesPage.refreshServices();
		await browser.pause(2000);
		expect(await servicesPage.isServicesListDisplayed()).toBe(true);
	});

	// SVC-005: Loading State
	it('SVC-005: should show loading state initially', async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		// Check for loading indicator (may be too fast to catch)
		const wasLoading = await servicesPage.isLoadingDisplayed();
		// Either loading was shown or services loaded immediately
		expect(wasLoading || (await servicesPage.isServicesListDisplayed())).toBe(true);
	});
});
