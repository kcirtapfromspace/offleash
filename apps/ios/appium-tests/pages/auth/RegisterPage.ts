import { BasePage } from '../BasePage';

export class RegisterPage extends BasePage {
	private firstNameField = 'register-first-name';
	private lastNameField = 'register-last-name';
	private emailField = 'register-email';
	private passwordField = 'register-password';
	private submitButton = 'register-submit-button';
	private verifyBanner = 'register-verify-banner';
	private errorBanner = 'auth-error-banner';
	private passwordStrength = 'password-strength-indicator';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.emailField);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.emailField, 5000);
	}

	async enterFirstName(name: string): Promise<void> {
		await this.type(this.firstNameField, name);
	}

	async enterLastName(name: string): Promise<void> {
		await this.type(this.lastNameField, name);
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

	async register(
		firstName: string,
		lastName: string,
		email: string,
		password: string
	): Promise<void> {
		await this.enterFirstName(firstName);
		await this.enterLastName(lastName);
		await this.enterEmail(email);
		await this.enterPassword(password);
		await this.tapSubmit();
	}

	async isVerifyBannerDisplayed(): Promise<boolean> {
		return this.isVisible(this.verifyBanner, 10000);
	}

	async isErrorDisplayed(): Promise<boolean> {
		return this.isVisible(this.errorBanner, 5000);
	}

	async isPasswordStrengthDisplayed(): Promise<boolean> {
		return this.isVisible(this.passwordStrength, 2000);
	}
}

export const registerPage = new RegisterPage();
