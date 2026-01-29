import { BasePage } from '../BasePage';

/**
 * Page object for the Invitation screens
 * Handles inviting walkers and customers to the organization
 */
export class InvitePage extends BasePage {
	// Selectors from Swift views
	private inviteWalkerSheet = 'invite-walker-sheet';
	private inviteCustomerSheet = 'invite-customer-sheet';
	private inviteEmailField = 'invite-email-field';
	private inviteNameField = 'invite-name-field';
	private inviteMessageField = 'invite-message-field';
	private sendButton = 'invite-send-button';
	private cancelButton = 'invite-cancel-button';
	private successMessage = 'invite-success-message';
	private errorMessage = 'invite-error-message';
	private inviteWalkerButton = 'settings-invite-walker';
	private inviteCustomerButton = 'settings-invite-customer';

	async waitForPageLoad(): Promise<void> {
		// Wait for either invite sheet to be visible
		try {
			await this.waitFor(this.inviteWalkerSheet, 5000);
		} catch {
			await this.waitFor(this.inviteCustomerSheet, 5000);
		}
	}

	async isDisplayed(): Promise<boolean> {
		return (
			(await this.isVisible(this.inviteWalkerSheet, 3000)) ||
			(await this.isVisible(this.inviteCustomerSheet, 3000))
		);
	}

	/**
	 * Open the walker invitation sheet from settings
	 */
	async openWalkerInviteSheet(): Promise<void> {
		await this.tap(this.inviteWalkerButton);
	}

	/**
	 * Open the customer invitation sheet from settings
	 */
	async openCustomerInviteSheet(): Promise<void> {
		await this.tap(this.inviteCustomerButton);
	}

	/**
	 * Wait for the walker invite sheet to be displayed
	 */
	async waitForWalkerInviteSheet(): Promise<void> {
		await this.waitFor(this.inviteWalkerSheet);
	}

	/**
	 * Wait for the customer invite sheet to be displayed
	 */
	async waitForCustomerInviteSheet(): Promise<void> {
		await this.waitFor(this.inviteCustomerSheet);
	}

	/**
	 * Check if walker invite sheet is displayed
	 */
	async isWalkerInviteSheetDisplayed(): Promise<boolean> {
		return this.isVisible(this.inviteWalkerSheet, 5000);
	}

	/**
	 * Check if customer invite sheet is displayed
	 */
	async isCustomerInviteSheetDisplayed(): Promise<boolean> {
		return this.isVisible(this.inviteCustomerSheet, 5000);
	}

	/**
	 * Enter the invitee's email address
	 */
	async enterInviteEmail(email: string): Promise<void> {
		await this.type(this.inviteEmailField, email);
	}

	/**
	 * Enter the invitee's name (optional)
	 */
	async enterInviteName(name: string): Promise<void> {
		await this.type(this.inviteNameField, name);
	}

	/**
	 * Enter a custom invitation message (optional)
	 */
	async enterInviteMessage(message: string): Promise<void> {
		await this.type(this.inviteMessageField, message);
	}

	/**
	 * Send the invitation
	 */
	async sendInvite(): Promise<void> {
		await this.tap(this.sendButton);
	}

	/**
	 * Cancel the invitation and close the sheet
	 */
	async cancelInvite(): Promise<void> {
		await this.tap(this.cancelButton);
	}

	/**
	 * Check if success message is displayed
	 */
	async isSuccessMessageDisplayed(): Promise<boolean> {
		return this.isVisible(this.successMessage, 5000);
	}

	/**
	 * Check if error message is displayed
	 */
	async isErrorMessageDisplayed(): Promise<boolean> {
		return this.isVisible(this.errorMessage, 5000);
	}

	/**
	 * Get the error message text
	 */
	async getErrorMessage(): Promise<string> {
		return this.getText(this.errorMessage);
	}

	/**
	 * Check if send button is enabled
	 */
	async isSendButtonEnabled(): Promise<boolean> {
		try {
			const element = await $(`~${this.sendButton}`);
			return await element.isEnabled();
		} catch {
			return false;
		}
	}

	/**
	 * Check if email field is displayed
	 */
	async isEmailFieldDisplayed(): Promise<boolean> {
		return this.isVisible(this.inviteEmailField, 5000);
	}

	/**
	 * Send a walker invitation with email
	 */
	async sendWalkerInvitation(email: string, name?: string, message?: string): Promise<void> {
		await this.openWalkerInviteSheet();
		await this.waitForWalkerInviteSheet();
		await this.enterInviteEmail(email);
		if (name) {
			await this.enterInviteName(name);
		}
		if (message) {
			await this.enterInviteMessage(message);
		}
		await this.sendInvite();
	}

	/**
	 * Send a customer invitation with email
	 */
	async sendCustomerInvitation(email: string, name?: string, message?: string): Promise<void> {
		await this.openCustomerInviteSheet();
		await this.waitForCustomerInviteSheet();
		await this.enterInviteEmail(email);
		if (name) {
			await this.enterInviteName(name);
		}
		if (message) {
			await this.enterInviteMessage(message);
		}
		await this.sendInvite();
	}

	/**
	 * Clear all fields in the invite form
	 */
	async clearForm(): Promise<void> {
		const emailField = await $(`~${this.inviteEmailField}`);
		if (await emailField.isExisting()) {
			await emailField.clearValue();
		}

		const nameField = await $(`~${this.inviteNameField}`);
		if (await nameField.isExisting()) {
			await nameField.clearValue();
		}

		const messageField = await $(`~${this.inviteMessageField}`);
		if (await messageField.isExisting()) {
			await messageField.clearValue();
		}
	}
}

export const invitePage = new InvitePage();
