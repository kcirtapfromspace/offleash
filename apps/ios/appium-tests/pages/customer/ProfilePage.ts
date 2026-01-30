import { BasePage } from '../BasePage';

export class ProfilePage extends BasePage {
	private profileHeader = 'profile-header';
	private editButton = 'profile-edit-button';
	private saveButton = 'profile-save-button';
	private logoutButton = 'logout-button';
	private petAddButton = 'pet-add-button';
	private locationsButton = 'profile-locations-button';
	private paymentsTab = 'payments-tab';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.profileHeader);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.profileHeader, 5000);
	}

	/**
	 * Navigate to profile by tapping the Profile tab in the tab bar
	 */
	async navigateToProfile(): Promise<void> {
		// In SwiftUI TabView, we need to tap the tab at its coordinates
		// The Profile tab is at x=287 (center of 240+94/2) based on page source
		const { width, height } = await browser.getWindowSize();

		// Profile tab is typically the third tab (rightmost)
		// Tab bar is at the bottom, calculate tap position
		const tabY = height - 40; // 40px from bottom
		const tabX = width * 0.75; // Right side of tab bar (3rd of 3 tabs)

		await browser
			.action('pointer')
			.move({ x: Math.round(tabX), y: Math.round(tabY) })
			.down()
			.pause(100)
			.up()
			.perform();

		// Wait for navigation to complete
		await browser.pause(2000);
	}

	async tapEdit(): Promise<void> {
		await this.tap(this.editButton);
	}

	async tapSave(): Promise<void> {
		await this.tap(this.saveButton);
	}

	/**
	 * Tap the logout button - scrolls down if needed since it's at the bottom of the profile
	 */
	async tapLogout(): Promise<void> {
		// Scroll to logout button which is at the bottom of the profile list
		await this.scrollToLogout();
		await this.tap(this.logoutButton);
	}

	/**
	 * Scroll down to find the logout button
	 */
	private async scrollToLogout(): Promise<void> {
		const maxScrolls = 5;
		for (let i = 0; i < maxScrolls; i++) {
			if (await this.isVisible(this.logoutButton, 1000)) {
				return;
			}
			await browser.execute('mobile: scroll', { direction: 'down' });
			await browser.pause(500);
		}
	}

	async tapAddPet(): Promise<void> {
		await this.tap(this.petAddButton);
	}

	async selectPet(petName: string): Promise<void> {
		await this.tap('pet-item-' + petName);
	}

	async tapLocations(): Promise<void> {
		await this.tap(this.locationsButton);
	}

	async tapPayments(): Promise<void> {
		await this.tap(this.paymentsTab);
	}

	/**
	 * Confirm the logout action in the alert dialog
	 */
	async confirmLogout(): Promise<void> {
		// Wait for the alert to appear
		await browser.pause(500);
		// Find and tap the "Log Out" button in the alert
		const logOutButton = await $(
			'-ios predicate string:label == "Log Out" AND type == "XCUIElementTypeButton"'
		);
		await logOutButton.waitForExist({ timeout: 10000 });
		await logOutButton.click();
	}
}

export const profilePage = new ProfilePage();
