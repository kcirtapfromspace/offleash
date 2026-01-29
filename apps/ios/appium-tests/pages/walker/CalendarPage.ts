import { BasePage } from '../BasePage';

export class CalendarPage extends BasePage {
	private calendarView = 'calendar-view';
	private toggleWeek = 'calendar-toggle-week';
	private bookingsList = 'calendar-bookings-list';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.calendarView);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.calendarView, 5000);
	}

	async toggleToWeekView(): Promise<void> {
		await this.tap(this.toggleWeek);
	}

	async selectDate(date: string): Promise<void> {
		await this.tap('calendar-day-' + date);
	}

	async isBookingsListDisplayed(): Promise<boolean> {
		return this.isVisible(this.bookingsList, 5000);
	}

	async getBookingsForSelectedDate(): Promise<number> {
		try {
			const list = await this.waitFor(this.bookingsList, 5000);
			const children = await list.$$('XCUIElementTypeCell');
			return children.length;
		} catch {
			return 0;
		}
	}
}

export const calendarPage = new CalendarPage();
