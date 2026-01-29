import { BasePage } from '../BasePage';

export class BookingFlowPage extends BasePage {
	private locationPicker = 'booking-location-picker';
	private addLocation = 'booking-add-location';
	private datePicker = 'booking-date-picker';
	private reviewSection = 'booking-review';
	private confirmButton = 'booking-confirm-button';
	private notesField = 'booking-notes-field';
	private cancelButton = 'booking-cancel-button';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.locationPicker);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.locationPicker, 5000);
	}

	async selectLocation(locationName?: string): Promise<void> {
		await this.tap(this.locationPicker);
		if (locationName) {
			await this.tap('location-item-' + locationName);
		}
	}

	async tapAddLocation(): Promise<void> {
		await this.tap(this.addLocation);
	}

	async selectDate(): Promise<void> {
		await this.tap(this.datePicker);
		// Select first available date
		await browser.pause(1000);
		await this.tap('confirm-button');
	}

	async selectTimeSlot(time: string): Promise<void> {
		await this.tap('booking-time-slot-' + time);
	}

	async enterNotes(notes: string): Promise<void> {
		await this.type(this.notesField, notes);
	}

	async isReviewDisplayed(): Promise<boolean> {
		return this.isVisible(this.reviewSection, 5000);
	}

	async tapConfirm(): Promise<void> {
		await this.tap(this.confirmButton);
	}

	async tapCancel(): Promise<void> {
		await this.tap(this.cancelButton);
	}

	async completeBookingFlow(notes?: string): Promise<void> {
		await this.selectLocation();
		await browser.pause(500);
		await this.selectDate();
		await browser.pause(500);
		// Select first available time slot
		const timeSlots = await $$('~booking-time-slot');
		if (timeSlots.length > 0) {
			await timeSlots[0].click();
		}
		if (notes) {
			await this.enterNotes(notes);
		}
		await this.tapConfirm();
	}
}

export const bookingFlowPage = new BookingFlowPage();
