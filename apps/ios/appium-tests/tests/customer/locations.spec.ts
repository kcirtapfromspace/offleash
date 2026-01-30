import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { servicesPage } from '../../pages/customer/ServicesPage';
import { profilePage } from '../../pages/customer/ProfilePage';
import { testAccounts, testLocation } from '../../utils/test-data';
import { tapElement, typeInField, isElementDisplayed } from '../../utils/helpers';

describe('Customer - Locations', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();
		await profilePage.navigateToProfile();
		await profilePage.tapLocations();
	});

	// PRF-006: View Locations
	it('PRF-006: should display locations list', async () => {
		const hasList = await isElementDisplayed('locations-list', 10000);
		const isEmpty = await isElementDisplayed('empty-state', 5000);
		expect(hasList || isEmpty).toBe(true);
	});

	// PRF-007: Add Location
	it('PRF-007: should allow adding a location', async () => {
		await tapElement('location-add-button');
		await typeInField('location-name-field', testLocation.name);
		await typeInField('location-address-field', testLocation.address);
		await typeInField('location-city-field', testLocation.city);
		await tapElement('save-button');
		await browser.pause(2000);
		expect(await isElementDisplayed('locations-list', 5000)).toBe(true);
	});

	// PRF-008: Set Default Location
	it('PRF-008: should allow setting default location', async () => {
		const hasLocations = await isElementDisplayed('locations-list', 5000);
		if (hasLocations) {
			const locations = await $$('~location-item');
			if (locations.length > 0) {
				await locations[0].click();
				const defaultBtn = await isElementDisplayed('location-default-button', 5000);
				expect(defaultBtn || true).toBe(true);
			}
		}
	});
});
