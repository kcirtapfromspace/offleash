import { BasePage } from '../BasePage';
import { pullToRefresh } from '../../utils/helpers';

export class ServicesPage extends BasePage {
	private tabServices = 'tab-services';
	private servicesList = 'services-list';
	private emptyState = 'empty-state';
	private loadingIndicator = 'loading-indicator';
	private errorBanner = 'error-banner';
	private retryButton = 'retry-button';

	/**
	 * Wait for the services page to load - accepts any valid state
	 * (services list, empty state, error, or loading)
	 */
	async waitForPageLoad(): Promise<void> {
		// Wait for the tab-services container to be visible first
		await this.waitFor(this.tabServices, 30000);
		// Then wait for any content state to appear
		await this.waitForAnyState();
	}

	/**
	 * Wait for any valid services page state
	 */
	private async waitForAnyState(): Promise<void> {
		const maxWait = 30000;
		const pollInterval = 500;
		let waited = 0;

		while (waited < maxWait) {
			// Check for services list (success with services)
			if (await this.isVisible(this.servicesList, 500)) {
				return;
			}
			// Check for empty state (success with no services)
			if (await this.isVisible(this.emptyState, 500)) {
				return;
			}
			// Check for error state
			if (await this.isVisible(this.errorBanner, 500)) {
				return;
			}
			// Check for loading indicator (still loading is ok)
			if (await this.isVisible(this.loadingIndicator, 500)) {
				// Wait a bit more for loading to finish
				await browser.pause(pollInterval);
				waited += pollInterval;
				continue;
			}
			// None of the expected states found, keep waiting
			await browser.pause(pollInterval);
			waited += pollInterval;
		}
	}

	async isDisplayed(): Promise<boolean> {
		// Check if we're on the services page by looking for the tab-services container
		// This is more reliable than checking for services-list which requires data
		return this.isVisible(this.tabServices, 30000);
	}

	async navigateToServices(): Promise<void> {
		await this.tap(this.tabServices);
	}

	async isServicesListDisplayed(): Promise<boolean> {
		return this.isVisible(this.servicesList, 10000);
	}

	async isEmptyStateDisplayed(): Promise<boolean> {
		return this.isVisible(this.emptyState, 5000);
	}

	async isLoadingDisplayed(): Promise<boolean> {
		return this.isVisible(this.loadingIndicator, 2000);
	}

	async isErrorDisplayed(): Promise<boolean> {
		return this.isVisible(this.errorBanner, 5000);
	}

	async tapRetry(): Promise<void> {
		await this.tap(this.retryButton);
	}

	async refreshServices(): Promise<void> {
		await pullToRefresh();
	}

	async selectService(serviceId: string): Promise<void> {
		await this.tap('service-card-' + serviceId);
	}

	async tapBookService(): Promise<void> {
		await this.tap('book-service-button');
	}

	async getServiceCount(): Promise<number> {
		const list = await this.waitFor(this.servicesList);
		const children = await list.$$('XCUIElementTypeCell');
		return children.length;
	}
}

export const servicesPage = new ServicesPage();
