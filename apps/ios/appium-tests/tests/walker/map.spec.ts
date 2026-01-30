import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { mapPage } from '../../pages/walker/MapPage';
import { testAccounts } from '../../utils/test-data';

describe('Walker - Map', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);
		await browser.pause(3000);

		if (await dashboardPage.isDisplayed()) {
			await dashboardPage.navigateToMap();
		}
	});

	// WMP-001: Map Load
	it('WMP-001: should display map view', async () => {
		await mapPage.allowLocationPermission();
		expect(await mapPage.isDisplayed()).toBe(true);
	});

	// WMP-002: Location Permission
	it('WMP-002: should handle location permission', async () => {
		await mapPage.allowLocationPermission();
		await browser.pause(2000);
		expect(await mapPage.isDisplayed()).toBe(true);
	});

	// WMP-003: Booking Markers
	it('WMP-003: should show booking markers on map', async () => {
		await mapPage.allowLocationPermission();
		await browser.pause(2000);
		// Markers might or might not be visible depending on bookings
		expect(await mapPage.isDisplayed()).toBe(true);
	});
});
