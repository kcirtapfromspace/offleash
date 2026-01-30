import { waitForElement, tapElement, typeInField, isElementDisplayed } from '../utils/helpers';

/**
 * Base page class with common functionality
 */
export class BasePage {
	/**
	 * Wait for the page to be loaded
	 */
	async waitForPageLoad(): Promise<void> {
		// Override in subclasses
	}

	/**
	 * Check if the page is displayed
	 */
	async isDisplayed(): Promise<boolean> {
		// Override in subclasses
		return true;
	}

	/**
	 * Navigate back
	 */
	async goBack(): Promise<void> {
		await tapElement('nav-back-button');
	}

	/**
	 * Wait for element by accessibility ID
	 */
	protected async waitFor(accessibilityId: string, timeout?: number) {
		return waitForElement(accessibilityId, timeout);
	}

	/**
	 * Tap element by accessibility ID
	 */
	protected async tap(accessibilityId: string) {
		return tapElement(accessibilityId);
	}

	/**
	 * Type text into field
	 */
	protected async type(accessibilityId: string, text: string) {
		return typeInField(accessibilityId, text);
	}

	/**
	 * Check if element is visible
	 */
	protected async isVisible(accessibilityId: string, timeout?: number) {
		return isElementDisplayed(accessibilityId, timeout);
	}

	/**
	 * Get element text
	 */
	protected async getText(accessibilityId: string): Promise<string> {
		const element = await this.waitFor(accessibilityId);
		return element.getText();
	}

	/**
	 * Check if element exists
	 */
	protected async exists(accessibilityId: string): Promise<boolean> {
		try {
			const element = await $('~' + accessibilityId);
			return await element.isExisting();
		} catch {
			return false;
		}
	}
}
