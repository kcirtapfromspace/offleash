import { BasePage } from '../BasePage';

export class LoginPage extends BasePage {
	// Selectors
	private emailField = 'login-email-field';
	private passwordField = 'login-password-field';
	private submitButton = 'login-submit-button';
	private googleButton = 'login-google-button';
	private registerLink = 'register-link';
	private errorBanner = 'auth-error-banner';
	private passwordStrength = 'password-strength-indicator';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.emailField);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.emailField, 5000);
	}

	async enterEmail(email: string): Promise<void> {
		await this.type(this.emailField, email);
	}

	async enterPassword(password: string): Promise<void> {
		await this.type(this.passwordField, password);
	}

	async tapSubmit(): Promise<void> {
		await this.tap(this.submitButton);
	}

	async tapGoogleSignIn(): Promise<void> {
		await this.tap(this.googleButton);
	}

	async tapRegister(): Promise<void> {
		await this.tap(this.registerLink);
	}

	async login(email: string, password: string): Promise<void> {
		await this.enterEmail(email);
		await this.enterPassword(password);
		await this.tapSubmit();
		// Wait for login to complete - either navigate away or show error
		await this.waitForLoginResult();
	}

	/**
	 * Wait for login result - either navigation away from login page or error displayed
	 */
	private async waitForLoginResult(): Promise<void> {
		const maxWait = 30000;
		const pollInterval = 500;
		let waited = 0;

		while (waited < maxWait) {
			// Check if we're still on login page
			const loginFormVisible = await this.isVisible(this.emailField, 500);
			if (!loginFormVisible) {
				// Successfully navigated away
				await browser.pause(1000); // Extra time for page to settle
				return;
			}

			// Check if error banner appeared
			const errorVisible = await this.isVisible(this.errorBanner, 500);
			if (errorVisible) {
				// Login failed with error
				return;
			}

			await browser.pause(pollInterval);
			waited += pollInterval;
		}
		// Timeout - continue anyway, test will fail if expected state not reached
	}

	async isErrorDisplayed(): Promise<boolean> {
		return this.isVisible(this.errorBanner, 5000);
	}

	async getErrorMessage(): Promise<string> {
		return this.getText(this.errorBanner);
	}

	async isPasswordStrengthDisplayed(): Promise<boolean> {
		return this.isVisible(this.passwordStrength, 2000);
	}
}

export const loginPage = new LoginPage();
