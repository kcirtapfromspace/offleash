import { BasePage } from '../BasePage';

export class MapPage extends BasePage {
	private mapView = 'map-view';
	private bookingMarkers = 'booking-markers';
	private permissionAllow = 'permission-allow-button';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.mapView);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.mapView, 5000);
	}

	async allowLocationPermission(): Promise<void> {
		try {
			if (await this.isVisible(this.permissionAllow, 3000)) {
				await this.tap(this.permissionAllow);
			}
		} catch {
			// Permission might already be granted
		}
	}

	async areMarkersDisplayed(): Promise<boolean> {
		return this.isVisible(this.bookingMarkers, 5000);
	}
}

export const mapPage = new MapPage();
