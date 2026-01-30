import { BasePage } from '../BasePage';

/**
 * Page object for the Subscriptions screen
 * Handles viewing and managing recurring service subscriptions
 */
export class SubscriptionsPage extends BasePage {
	// Selectors from Swift views
	private subscriptionsList = 'subscriptions-list';
	private subscriptionsButton = 'subscriptions-button';
	private subscriptionDetail = 'subscription-detail';
	private subscriptionStatus = 'subscription-status';
	private subscriptionFrequency = 'subscription-frequency';
	private subscriptionNextDate = 'subscription-next-date';
	private subscriptionPauseButton = 'subscription-pause-button';
	private subscriptionCancelButton = 'subscription-cancel-button';
	private subscriptionCancelConfirm = 'subscription-cancel-confirm';
	private emptyState = 'empty-state';

	async waitForPageLoad(): Promise<void> {
		// Either list or empty state should be visible
		try {
			await this.waitFor(this.subscriptionsList, 5000);
		} catch {
			await this.waitFor(this.emptyState, 5000);
		}
	}

	async isDisplayed(): Promise<boolean> {
		return (
			(await this.isVisible(this.subscriptionsList, 5000)) ||
			(await this.isVisible(this.emptyState, 2000))
		);
	}

	/**
	 * Navigate to subscriptions from payments tab
	 */
	async navigateToSubscriptions(): Promise<void> {
		await this.tap(this.subscriptionsButton);
	}

	/**
	 * Get the count of active subscriptions
	 */
	async getSubscriptionCount(): Promise<number> {
		try {
			const list = await this.waitFor(this.subscriptionsList, 5000);
			const children = await list.$$('XCUIElementTypeCell');
			return children.length;
		} catch {
			return 0;
		}
	}

	/**
	 * Select a subscription from the list by ID
	 */
	async selectSubscription(subscriptionId: string): Promise<void> {
		await this.tap(`subscription-item-${subscriptionId}`);
	}

	/**
	 * Select the first subscription in the list
	 */
	async selectFirstSubscription(): Promise<void> {
		const subscriptions = await $$('~subscription-item');
		if (subscriptions.length > 0) {
			await subscriptions[0].click();
		}
	}

	/**
	 * Check if subscription detail view is displayed
	 */
	async isSubscriptionDetailDisplayed(): Promise<boolean> {
		return this.isVisible(this.subscriptionDetail, 5000);
	}

	/**
	 * Get the subscription status text
	 */
	async getSubscriptionStatus(): Promise<string> {
		return this.getText(this.subscriptionStatus);
	}

	/**
	 * Get the subscription frequency text
	 */
	async getSubscriptionFrequency(): Promise<string> {
		return this.getText(this.subscriptionFrequency);
	}

	/**
	 * Get the next scheduled date
	 */
	async getNextScheduledDate(): Promise<string> {
		return this.getText(this.subscriptionNextDate);
	}

	/**
	 * Pause the subscription
	 */
	async pauseSubscription(): Promise<void> {
		await this.tap(this.subscriptionPauseButton);
	}

	/**
	 * Cancel the subscription
	 */
	async cancelSubscription(): Promise<void> {
		await this.tap(this.subscriptionCancelButton);
	}

	/**
	 * Confirm subscription cancellation
	 */
	async confirmCancellation(): Promise<void> {
		await this.tap(this.subscriptionCancelConfirm);
	}

	/**
	 * Check if empty state is displayed (no subscriptions)
	 */
	async isEmptyStateDisplayed(): Promise<boolean> {
		return this.isVisible(this.emptyState, 5000);
	}

	/**
	 * Check if subscription status shows as active
	 */
	async isSubscriptionActive(): Promise<boolean> {
		try {
			const status = await this.getSubscriptionStatus();
			return status.toLowerCase().includes('active');
		} catch {
			return false;
		}
	}

	/**
	 * Check if subscription status shows as paused
	 */
	async isSubscriptionPaused(): Promise<boolean> {
		try {
			const status = await this.getSubscriptionStatus();
			return status.toLowerCase().includes('paused');
		} catch {
			return false;
		}
	}
}

export const subscriptionsPage = new SubscriptionsPage();
