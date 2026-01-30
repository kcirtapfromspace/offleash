import { BasePage } from '../BasePage';

export class BookingRequestsPage extends BasePage {
	private requestsList = 'pending-requests-list';
	private acceptButton = 'accept-booking-button';
	private declineButton = 'decline-booking-button';
	private declineReasonField = 'decline-reason-field';
	private declineSubmit = 'decline-submit';
	private emptyState = 'empty-state';
	private requestDetail = 'request-detail';

	async waitForPageLoad(): Promise<void> {
		// Either list or empty state should be visible
		try {
			await this.waitFor(this.requestsList, 5000);
		} catch {
			await this.waitFor(this.emptyState, 5000);
		}
	}

	async isDisplayed(): Promise<boolean> {
		return (
			(await this.isVisible(this.requestsList, 5000)) ||
			(await this.isVisible(this.emptyState, 2000))
		);
	}

	async isEmptyStateDisplayed(): Promise<boolean> {
		return this.isVisible(this.emptyState, 5000);
	}

	async selectRequest(requestId: string): Promise<void> {
		await this.tap('request-item-' + requestId);
	}

	async isRequestDetailDisplayed(): Promise<boolean> {
		return this.isVisible(this.requestDetail, 5000);
	}

	async tapAccept(): Promise<void> {
		await this.tap(this.acceptButton);
	}

	async tapDecline(): Promise<void> {
		await this.tap(this.declineButton);
	}

	async enterDeclineReason(reason: string): Promise<void> {
		await this.type(this.declineReasonField, reason);
	}

	async submitDecline(): Promise<void> {
		await this.tap(this.declineSubmit);
	}

	async declineWithReason(reason: string): Promise<void> {
		await this.tapDecline();
		await this.enterDeclineReason(reason);
		await this.submitDecline();
	}

	async getRequestCount(): Promise<number> {
		try {
			const list = await this.waitFor(this.requestsList, 5000);
			const children = await list.$$('XCUIElementTypeCell');
			return children.length;
		} catch {
			return 0;
		}
	}
}

export const bookingRequestsPage = new BookingRequestsPage();
