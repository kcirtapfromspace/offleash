import { BasePage } from '../BasePage';

export class ProfilePage extends BasePage {
	private tabProfile = 'tab-profile';
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

	async navigateToProfile(): Promise<void> {
		await this.tap(this.tabProfile);
	}

	async tapEdit(): Promise<void> {
		await this.tap(this.editButton);
	}

	async tapSave(): Promise<void> {
		await this.tap(this.saveButton);
	}

	async tapLogout(): Promise<void> {
		await this.tap(this.logoutButton);
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

	async confirmLogout(): Promise<void> {
		await this.tap('confirm-button');
	}
}

export const profilePage = new ProfilePage();
