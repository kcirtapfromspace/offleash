import { BasePage } from '../BasePage';

/**
 * Page object for the Service Areas configuration screen
 * Handles setting geographic service area for walkers
 */
export class ServiceAreasPage extends BasePage {
	// Selectors from Swift views
	private serviceAreasView = 'settings-service-areas';
	private mapView = 'map-view';
	private saveButton = 'service-areas-save';
	private cancelButton = 'service-areas-cancel';
	private resetButton = 'service-areas-reset';
	private radiusSlider = 'service-radius-slider';
	private radiusLabel = 'service-radius-label';
	private currentLocation = 'current-location-button';
	private permissionAllow = 'permission-allow-button';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.serviceAreasView);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.serviceAreasView, 5000);
	}

	/**
	 * Navigate to service areas settings
	 */
	async navigateToServiceAreas(): Promise<void> {
		await this.tap(this.serviceAreasView);
	}

	/**
	 * Check if the map view is displayed
	 */
	async isMapDisplayed(): Promise<boolean> {
		return this.isVisible(this.mapView, 5000);
	}

	/**
	 * Save the service area configuration
	 */
	async saveServiceArea(): Promise<void> {
		await this.tap(this.saveButton);
	}

	/**
	 * Cancel changes and go back
	 */
	async cancelChanges(): Promise<void> {
		await this.tap(this.cancelButton);
	}

	/**
	 * Reset to default service area
	 */
	async resetToDefault(): Promise<void> {
		await this.tap(this.resetButton);
	}

	/**
	 * Adjust the service radius using the slider
	 */
	async adjustServiceRadius(percentage: number): Promise<void> {
		const slider = await $(`~${this.radiusSlider}`);
		if (await slider.isExisting()) {
			// Get slider dimensions
			const location = await slider.getLocation();
			const size = await slider.getSize();

			// Calculate target position based on percentage
			const targetX = location.x + size.width * (percentage / 100);
			const targetY = location.y + size.height / 2;

			// Perform drag to position
			await browser
				.action('pointer')
				.move({ x: Math.round(targetX), y: Math.round(targetY) })
				.down()
				.up()
				.perform();
		}
	}

	/**
	 * Get the current radius value displayed
	 */
	async getCurrentRadius(): Promise<string> {
		return this.getText(this.radiusLabel);
	}

	/**
	 * Center map on current location
	 */
	async centerOnCurrentLocation(): Promise<void> {
		await this.tap(this.currentLocation);
	}

	/**
	 * Allow location permission if prompted
	 */
	async allowLocationPermission(): Promise<void> {
		try {
			if (await this.isVisible(this.permissionAllow, 3000)) {
				await this.tap(this.permissionAllow);
			}
		} catch {
			// Permission might already be granted
		}
	}

	/**
	 * Check if save button is enabled
	 */
	async isSaveButtonEnabled(): Promise<boolean> {
		try {
			const element = await $(`~${this.saveButton}`);
			return await element.isEnabled();
		} catch {
			return false;
		}
	}

	/**
	 * Pinch to zoom on the map
	 */
	async zoomIn(): Promise<void> {
		const map = await $(`~${this.mapView}`);
		if (await map.isExisting()) {
			const location = await map.getLocation();
			const size = await map.getSize();
			const centerX = location.x + size.width / 2;
			const centerY = location.y + size.height / 2;

			// Perform pinch out gesture to zoom in
			await browser.execute('mobile: pinch', {
				scale: 1.5,
				velocity: 1,
				x: centerX,
				y: centerY,
			});
		}
	}

	/**
	 * Pinch to zoom out on the map
	 */
	async zoomOut(): Promise<void> {
		const map = await $(`~${this.mapView}`);
		if (await map.isExisting()) {
			const location = await map.getLocation();
			const size = await map.getSize();
			const centerX = location.x + size.width / 2;
			const centerY = location.y + size.height / 2;

			// Perform pinch in gesture to zoom out
			await browser.execute('mobile: pinch', {
				scale: 0.5,
				velocity: 1,
				x: centerX,
				y: centerY,
			});
		}
	}

	/**
	 * Pan the map in a direction
	 */
	async panMap(direction: 'up' | 'down' | 'left' | 'right'): Promise<void> {
		const map = await $(`~${this.mapView}`);
		if (await map.isExisting()) {
			const location = await map.getLocation();
			const size = await map.getSize();
			const centerX = location.x + size.width / 2;
			const centerY = location.y + size.height / 2;

			let endX = centerX;
			let endY = centerY;
			const offset = 100;

			switch (direction) {
				case 'up':
					endY = centerY + offset;
					break;
				case 'down':
					endY = centerY - offset;
					break;
				case 'left':
					endX = centerX + offset;
					break;
				case 'right':
					endX = centerX - offset;
					break;
			}

			await browser
				.action('pointer')
				.move({ x: Math.round(centerX), y: Math.round(centerY) })
				.down()
				.move({ x: Math.round(endX), y: Math.round(endY), duration: 300 })
				.up()
				.perform();
		}
	}
}

export const serviceAreasPage = new ServiceAreasPage();
