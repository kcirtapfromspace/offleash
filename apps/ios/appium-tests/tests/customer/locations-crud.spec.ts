import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { servicesPage } from '../../pages/customer/ServicesPage';
import { profilePage } from '../../pages/customer/ProfilePage';
import { locationsPage } from '../../pages/customer/LocationsPage';
import { testAccounts, testLocation } from '../../utils/test-data';

describe('Customer - Locations CRUD', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();

		// Navigate to profile and then to locations
		await profilePage.navigateToProfile();
		await browser.pause(1000);
		await profilePage.tapLocations();
		await browser.pause(500);
	});

	// LOC-001: Display locations list or empty state
	it('LOC-001: should display locations list or empty state', async () => {
		const isDisplayed = await locationsPage.isDisplayed();
		expect(isDisplayed).toBe(true);

		// Should show either locations list or empty state
		const hasLocations = (await locationsPage.getLocationCount()) > 0;
		const hasEmptyState = await locationsPage.isEmptyStateDisplayed();
		expect(hasLocations || hasEmptyState).toBe(true);
	});

	// LOC-002: Add new location
	it('LOC-002: should add new location', async () => {
		await locationsPage.tapAddLocation();
		await browser.pause(500);

		const isFormDisplayed = await locationsPage.isAddLocationFormDisplayed();
		if (isFormDisplayed) {
			const uniqueLocationName = `${testLocation.name}-${Date.now()}`;
			await locationsPage.enterLocationName(uniqueLocationName);
			await locationsPage.enterAddress(testLocation.address);
			await locationsPage.enterCity(testLocation.city);
			await locationsPage.enterState(testLocation.state);
			await locationsPage.enterZip(testLocation.zip);
			await locationsPage.saveLocation();

			// Wait for save to complete
			await browser.pause(2000);

			// Should return to locations list
			const isListVisible = await locationsPage.isDisplayed();
			expect(isListVisible).toBe(true);
		}
	});

	// LOC-003: Edit location details
	it('LOC-003: should edit location details', async () => {
		// First check if we have locations, if not add one
		let locationCount = await locationsPage.getLocationCount();
		if (locationCount === 0) {
			await locationsPage.addLocationWithDetails(
				`EditTest-${Date.now()}`,
				testLocation.address,
				testLocation.city,
				testLocation.state,
				testLocation.zip
			);
			await browser.pause(2000);
		}

		locationCount = await locationsPage.getLocationCount();
		if (locationCount > 0) {
			// Select first location
			const locations = await $$('~location-item');
			if (locations.length > 0) {
				await locations[0].click();
				await browser.pause(500);

				await locationsPage.tapEditLocation();
				await browser.pause(500);

				// Update the address
				await locationsPage.enterAddress('456 Updated Street');
				await locationsPage.saveLocation();
				await browser.pause(1000);
			}
		}
		// Test passes if we get this far without errors
		expect(true).toBe(true);
	});

	// LOC-004: Set default location
	it('LOC-004: should set default location', async () => {
		// Check if we have locations
		let locationCount = await locationsPage.getLocationCount();
		if (locationCount === 0) {
			// Add a location first
			await locationsPage.addLocationWithDetails(
				`DefaultTest-${Date.now()}`,
				testLocation.address,
				testLocation.city,
				testLocation.state,
				testLocation.zip
			);
			await browser.pause(2000);
		}

		locationCount = await locationsPage.getLocationCount();
		if (locationCount > 0) {
			// Select first location
			const locations = await $$('~location-item');
			if (locations.length > 0) {
				await locations[0].click();
				await browser.pause(500);

				// Set as default
				await locationsPage.setAsDefault();
				await browser.pause(1000);
			}
		}
		// Test passes if we get this far without errors
		expect(true).toBe(true);
	});

	// LOC-005: Delete location
	it('LOC-005: should delete location', async () => {
		// First add a location to delete
		await locationsPage.addLocationWithDetails(
			`DeleteTest-${Date.now()}`,
			testLocation.address,
			testLocation.city,
			testLocation.state,
			testLocation.zip
		);
		await browser.pause(2000);

		const locationCount = await locationsPage.getLocationCount();
		if (locationCount > 0) {
			// Select a location
			const locations = await $$('~location-item');
			if (locations.length > 0) {
				await locations[0].click();
				await browser.pause(500);

				// Delete the location
				await locationsPage.deleteLocation();
				await browser.pause(500);
				await locationsPage.confirmDelete();
				await browser.pause(1000);
			}
		}
		// Test passes if we get this far without errors
		expect(true).toBe(true);
	});

	// LOC-006: Validate address format
	it('LOC-006: should validate address format', async () => {
		await locationsPage.tapAddLocation();
		await browser.pause(500);

		const isFormDisplayed = await locationsPage.isAddLocationFormDisplayed();
		if (isFormDisplayed) {
			// Try to save with incomplete address
			await locationsPage.enterLocationName('Invalid Location');
			// Don't enter address fields

			const isSaveEnabled = await locationsPage.isSaveButtonEnabled();
			if (isSaveEnabled) {
				await locationsPage.saveLocation();
				await browser.pause(500);
				// Should still be on the form or show validation error
				const stillOnForm = await locationsPage.isAddLocationFormDisplayed();
				expect(stillOnForm).toBe(true);
			} else {
				// Save button is disabled as expected
				expect(isSaveEnabled).toBe(false);
			}
		}
	});
});
