import { BasePage } from '../BasePage';

/**
 * Page object for the Locations management screen
 * Handles location CRUD operations
 */
export class LocationsPage extends BasePage {
	// Selectors from Swift views
	private locationsView = 'my-locations-view';
	private locationsList = 'locations-list';
	private locationAddButton = 'location-add-button';
	private locationNameField = 'location-name-field';
	private locationAddressField = 'location-address-field';
	private locationCityField = 'location-city-field';
	private locationStateField = 'location-state-field';
	private locationZipField = 'location-zip-field';
	private locationSaveButton = 'location-save-button';
	private locationDeleteButton = 'location-delete-button';
	private locationDeleteConfirm = 'location-delete-confirm';
	private locationDefaultButton = 'location-default-button';
	private locationEditButton = 'location-edit-button';
	private locationCancelButton = 'location-cancel-button';
	private emptyState = 'empty-state';

	async waitForPageLoad(): Promise<void> {
		// Either list or empty state should be visible
		try {
			await this.waitFor(this.locationsList, 5000);
		} catch {
			try {
				await this.waitFor(this.locationsView, 5000);
			} catch {
				await this.waitFor(this.emptyState, 5000);
			}
		}
	}

	async isDisplayed(): Promise<boolean> {
		return (
			(await this.isVisible(this.locationsList, 5000)) ||
			(await this.isVisible(this.locationsView, 3000)) ||
			(await this.isVisible(this.emptyState, 2000))
		);
	}

	/**
	 * Tap the add location button
	 */
	async tapAddLocation(): Promise<void> {
		await this.tap(this.locationAddButton);
	}

	/**
	 * Enter the location name/label
	 */
	async enterLocationName(name: string): Promise<void> {
		await this.type(this.locationNameField, name);
	}

	/**
	 * Enter the street address
	 */
	async enterAddress(address: string): Promise<void> {
		await this.type(this.locationAddressField, address);
	}

	/**
	 * Enter the city
	 */
	async enterCity(city: string): Promise<void> {
		await this.type(this.locationCityField, city);
	}

	/**
	 * Enter the state
	 */
	async enterState(state: string): Promise<void> {
		await this.type(this.locationStateField, state);
	}

	/**
	 * Enter the zip code
	 */
	async enterZip(zip: string): Promise<void> {
		await this.type(this.locationZipField, zip);
	}

	/**
	 * Save the location
	 */
	async saveLocation(): Promise<void> {
		await this.tap(this.locationSaveButton);
	}

	/**
	 * Select a location from the list by name
	 */
	async selectLocation(locationName: string): Promise<void> {
		await this.tap(`location-item-${locationName}`);
	}

	/**
	 * Tap edit button on selected location
	 */
	async tapEditLocation(): Promise<void> {
		await this.tap(this.locationEditButton);
	}

	/**
	 * Set the selected location as default
	 */
	async setAsDefault(): Promise<void> {
		await this.tap(this.locationDefaultButton);
	}

	/**
	 * Delete the selected location
	 */
	async deleteLocation(): Promise<void> {
		await this.tap(this.locationDeleteButton);
	}

	/**
	 * Confirm location deletion
	 */
	async confirmDelete(): Promise<void> {
		await this.tap(this.locationDeleteConfirm);
	}

	/**
	 * Cancel add/edit location flow
	 */
	async cancelAddLocation(): Promise<void> {
		await this.tap(this.locationCancelButton);
	}

	/**
	 * Get the count of locations in the list
	 */
	async getLocationCount(): Promise<number> {
		try {
			const list = await this.waitFor(this.locationsList, 5000);
			const children = await list.$$('XCUIElementTypeCell');
			return children.length;
		} catch {
			return 0;
		}
	}

	/**
	 * Check if empty state is displayed (no locations)
	 */
	async isEmptyStateDisplayed(): Promise<boolean> {
		return this.isVisible(this.emptyState, 5000);
	}

	/**
	 * Check if add location form is displayed
	 */
	async isAddLocationFormDisplayed(): Promise<boolean> {
		return this.isVisible(this.locationNameField, 5000);
	}

	/**
	 * Check if save button is enabled
	 */
	async isSaveButtonEnabled(): Promise<boolean> {
		try {
			const element = await $(`~${this.locationSaveButton}`);
			return await element.isEnabled();
		} catch {
			return false;
		}
	}

	/**
	 * Add a new location with all fields
	 */
	async addLocationWithDetails(
		name: string,
		address: string,
		city: string,
		state: string,
		zip: string
	): Promise<void> {
		await this.tapAddLocation();
		await browser.pause(500);
		await this.enterLocationName(name);
		await this.enterAddress(address);
		await this.enterCity(city);
		await this.enterState(state);
		await this.enterZip(zip);
		await this.saveLocation();
	}
}

export const locationsPage = new LocationsPage();
