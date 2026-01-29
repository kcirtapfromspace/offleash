/**
 * Wait for an element to be displayed
 */
export async function waitForElement(
	selector: string,
	timeout: number = 30000
): Promise<WebdriverIO.Element> {
	const element = await $(`~${selector}`);
	await element.waitForDisplayed({ timeout });
	return element;
}

/**
 * Wait for an element to be clickable and then click it
 */
export async function tapElement(selector: string, timeout: number = 30000): Promise<void> {
	const element = await waitForElement(selector, timeout);
	await element.click();
}

/**
 * Type text into an input field
 */
export async function typeInField(selector: string, text: string): Promise<void> {
	const element = await waitForElement(selector);
	await element.clearValue();
	await element.setValue(text);
}

/**
 * Get text from an element
 */
export async function getElementText(selector: string): Promise<string> {
	const element = await waitForElement(selector);
	return element.getText();
}

/**
 * Check if an element is displayed
 */
export async function isElementDisplayed(
	selector: string,
	timeout: number = 5000
): Promise<boolean> {
	try {
		const element = await $(`~${selector}`);
		await element.waitForDisplayed({ timeout });
		return true;
	} catch {
		return false;
	}
}

/**
 * Wait for element to disappear
 */
export async function waitForElementToDisappear(
	selector: string,
	timeout: number = 30000
): Promise<void> {
	const element = await $(`~${selector}`);
	await element.waitForDisplayed({ timeout, reverse: true });
}

/**
 * Scroll down until element is visible
 */
export async function scrollToElement(selector: string, maxScrolls: number = 5): Promise<void> {
	for (let i = 0; i < maxScrolls; i++) {
		if (await isElementDisplayed(selector, 1000)) {
			return;
		}
		await browser.execute('mobile: scroll', { direction: 'down' });
	}
	throw new Error(`Element ${selector} not found after ${maxScrolls} scrolls`);
}

/**
 * Take a screenshot with a custom name
 */
export async function takeNamedScreenshot(name: string): Promise<void> {
	const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
	await browser.saveScreenshot(`./screenshots/${name}-${timestamp}.png`);
}

/**
 * Wait for app to be ready (e.g., after launch or navigation)
 */
export async function waitForAppReady(timeout: number = 10000): Promise<void> {
	await browser.pause(1000);
	// Wait for any loading indicators to disappear
	try {
		await waitForElementToDisappear('loading-indicator', timeout);
	} catch {
		// Loading indicator might not be present
	}
}

/**
 * Pull to refresh on a list
 */
export async function pullToRefresh(): Promise<void> {
	const { width, height } = await browser.getWindowSize();
	await browser.execute('mobile: scroll', {
		direction: 'down',
		velocity: 1500,
	});
}

/**
 * Dismiss keyboard if visible
 */
export async function dismissKeyboard(): Promise<void> {
	try {
		await browser.execute('mobile: hideKeyboard');
	} catch {
		// Keyboard might not be visible
	}
}

/**
 * Assert element contains text
 */
export async function assertElementContainsText(
	selector: string,
	expectedText: string
): Promise<void> {
	const element = await waitForElement(selector);
	const actualText = await element.getText();
	if (!actualText.includes(expectedText)) {
		throw new Error(`Expected "${selector}" to contain "${expectedText}" but got "${actualText}"`);
	}
}
