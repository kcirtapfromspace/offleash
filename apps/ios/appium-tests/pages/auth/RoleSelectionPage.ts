import { BasePage } from '../BasePage';

export class RoleSelectionPage extends BasePage {
	// Selectors
	private customerButton = 'role-selection-customer-button';
	private walkerButton = 'role-selection-walker-button';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.customerButton);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.customerButton, 5000);
	}

	async selectCustomerRole(): Promise<void> {
		await this.tap(this.customerButton);
	}

	async selectWalkerRole(): Promise<void> {
		await this.tap(this.walkerButton);
	}
}

export const roleSelectionPage = new RoleSelectionPage();
