import { BasePage } from '../BasePage';
import { pullToRefresh } from '../../utils/helpers';

export class ServicesPage extends BasePage {
	private tabServices = 'tab-services';
	private servicesList = 'services-list';
	private emptyState = 'empty-state';
	private loadingIndicator = 'loading-indicator';
	private errorBanner = 'error-banner';
	private retryButton = 'retry-button';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.servicesList, 30000);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.servicesList, 5000);
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
