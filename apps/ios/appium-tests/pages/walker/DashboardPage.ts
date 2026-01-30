import { BasePage } from '../BasePage';

export class DashboardPage extends BasePage {
	private dashboard = 'walker-dashboard';
	private onDutyToggle = 'on-duty-toggle';
	private todayBookingsCount = 'today-bookings-count';
	private weeklyEarnings = 'weekly-earnings';
	private performanceMetrics = 'performance-metrics';
	private tabRequests = 'tab-requests';
	private tabCalendar = 'tab-calendar';
	private tabMap = 'tab-map';
	private tabSettings = 'tab-settings';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.dashboard);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.dashboard, 5000);
	}

	async toggleOnDuty(): Promise<void> {
		await this.tap(this.onDutyToggle);
	}

	async getTodayBookingsCount(): Promise<string> {
		return this.getText(this.todayBookingsCount);
	}

	async getWeeklyEarnings(): Promise<string> {
		return this.getText(this.weeklyEarnings);
	}

	async isPerformanceMetricsDisplayed(): Promise<boolean> {
		return this.isVisible(this.performanceMetrics, 5000);
	}

	async navigateToRequests(): Promise<void> {
		await this.tap(this.tabRequests);
	}

	async navigateToCalendar(): Promise<void> {
		await this.tap(this.tabCalendar);
	}

	async navigateToMap(): Promise<void> {
		await this.tap(this.tabMap);
	}

	async navigateToSettings(): Promise<void> {
		await this.tap(this.tabSettings);
	}
}

export const dashboardPage = new DashboardPage();
