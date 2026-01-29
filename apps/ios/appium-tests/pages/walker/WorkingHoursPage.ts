import { BasePage } from '../BasePage';

/**
 * Page object for the Working Hours configuration screen
 * Handles setting availability schedule for walkers
 */
export class WorkingHoursPage extends BasePage {
	// Selectors from Swift views
	private workingHoursView = 'settings-working-hours';
	private saveButton = 'working-hours-save';
	private cancelButton = 'working-hours-cancel';
	private resetButton = 'working-hours-reset';

	// Day toggles - dynamic selectors
	private dayToggle = (day: string) => `working-hours-${day.toLowerCase()}-toggle`;
	private dayStartTime = (day: string) => `working-hours-${day.toLowerCase()}-start`;
	private dayEndTime = (day: string) => `working-hours-${day.toLowerCase()}-end`;

	// Days of the week
	private days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.workingHoursView);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.workingHoursView, 5000);
	}

	/**
	 * Navigate to working hours settings
	 */
	async navigateToWorkingHours(): Promise<void> {
		await this.tap(this.workingHoursView);
	}

	/**
	 * Toggle availability for a specific day
	 */
	async toggleDay(day: string): Promise<void> {
		await this.tap(this.dayToggle(day));
	}

	/**
	 * Check if a specific day is enabled
	 */
	async isDayEnabled(day: string): Promise<boolean> {
		try {
			const element = await $(`~${this.dayToggle(day)}`);
			const value = await element.getAttribute('value');
			return value === '1' || value === 'true';
		} catch {
			return false;
		}
	}

	/**
	 * Set the start time for a specific day
	 */
	async setStartTime(day: string, time: string): Promise<void> {
		await this.tap(this.dayStartTime(day));
		await browser.pause(500);
		// Handle time picker - this may need to use iOS date picker interactions
		await this.selectTime(time);
	}

	/**
	 * Set the end time for a specific day
	 */
	async setEndTime(day: string, time: string): Promise<void> {
		await this.tap(this.dayEndTime(day));
		await browser.pause(500);
		// Handle time picker
		await this.selectTime(time);
	}

	/**
	 * Helper to select time in iOS time picker
	 */
	private async selectTime(time: string): Promise<void> {
		// For iOS time picker, we typically need to use wheel selectors
		// This is a simplified version - may need adjustment based on actual picker UI
		const confirmButton = await $('~confirm-button');
		if (await confirmButton.isExisting()) {
			await confirmButton.click();
		}
	}

	/**
	 * Save the working hours configuration
	 */
	async saveWorkingHours(): Promise<void> {
		await this.tap(this.saveButton);
	}

	/**
	 * Cancel changes and go back
	 */
	async cancelChanges(): Promise<void> {
		await this.tap(this.cancelButton);
	}

	/**
	 * Reset to default working hours
	 */
	async resetToDefault(): Promise<void> {
		await this.tap(this.resetButton);
	}

	/**
	 * Get list of all enabled days
	 */
	async getEnabledDays(): Promise<string[]> {
		const enabledDays: string[] = [];
		for (const day of this.days) {
			if (await this.isDayEnabled(day)) {
				enabledDays.push(day);
			}
		}
		return enabledDays;
	}

	/**
	 * Enable all weekdays (Mon-Fri)
	 */
	async enableWeekdays(): Promise<void> {
		const weekdays = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday'];
		for (const day of weekdays) {
			if (!(await this.isDayEnabled(day))) {
				await this.toggleDay(day);
			}
		}
	}

	/**
	 * Disable all days
	 */
	async disableAllDays(): Promise<void> {
		for (const day of this.days) {
			if (await this.isDayEnabled(day)) {
				await this.toggleDay(day);
			}
		}
	}

	/**
	 * Check if save button is enabled
	 */
	async isSaveButtonEnabled(): Promise<boolean> {
		try {
			const element = await $(`~${this.saveButton}`);
			return await element.isEnabled();
		} catch {
			return false;
		}
	}

	/**
	 * Configure working hours for a specific day
	 */
	async configureDay(day: string, startTime: string, endTime: string): Promise<void> {
		// Enable the day if not already enabled
		if (!(await this.isDayEnabled(day))) {
			await this.toggleDay(day);
		}
		await browser.pause(300);
		await this.setStartTime(day, startTime);
		await browser.pause(300);
		await this.setEndTime(day, endTime);
	}
}

export const workingHoursPage = new WorkingHoursPage();
