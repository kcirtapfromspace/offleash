import { BasePage } from '../BasePage';

/**
 * Page object for the Walker Profile update screen
 * Handles updating walker-specific profile information
 */
export class WalkerProfilePage extends BasePage {
	// Selectors from Swift views
	private profileUpdateView = 'settings-profile-update';
	private bioField = 'walker-bio-field';
	private experienceField = 'walker-experience-field';
	private certificationsList = 'walker-certifications-list';
	private addCertificationButton = 'add-certification-button';
	private photoUploadButton = 'walker-photo-upload';
	private saveButton = 'profile-save-button';
	private cancelButton = 'profile-cancel-button';
	private profileHeader = 'walker-profile-header';
	private ratingDisplay = 'walker-rating-display';
	private reviewsCount = 'walker-reviews-count';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.profileUpdateView);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.profileUpdateView, 5000);
	}

	/**
	 * Navigate to profile update settings
	 */
	async navigateToProfileUpdate(): Promise<void> {
		await this.tap(this.profileUpdateView);
	}

	/**
	 * Enter or update the walker bio
	 */
	async enterBio(bio: string): Promise<void> {
		await this.type(this.bioField, bio);
	}

	/**
	 * Get the current bio text
	 */
	async getBio(): Promise<string> {
		return this.getText(this.bioField);
	}

	/**
	 * Enter or update experience information
	 */
	async enterExperience(experience: string): Promise<void> {
		await this.type(this.experienceField, experience);
	}

	/**
	 * Get the current experience text
	 */
	async getExperience(): Promise<string> {
		return this.getText(this.experienceField);
	}

	/**
	 * Save the profile changes
	 */
	async saveProfile(): Promise<void> {
		await this.tap(this.saveButton);
	}

	/**
	 * Cancel changes and go back
	 */
	async cancelChanges(): Promise<void> {
		await this.tap(this.cancelButton);
	}

	/**
	 * Tap to upload/change profile photo
	 */
	async tapUploadPhoto(): Promise<void> {
		await this.tap(this.photoUploadButton);
	}

	/**
	 * Tap to add a certification
	 */
	async tapAddCertification(): Promise<void> {
		await this.tap(this.addCertificationButton);
	}

	/**
	 * Check if certifications list is displayed
	 */
	async isCertificationsListDisplayed(): Promise<boolean> {
		return this.isVisible(this.certificationsList, 5000);
	}

	/**
	 * Get the number of certifications
	 */
	async getCertificationsCount(): Promise<number> {
		try {
			const list = await this.waitFor(this.certificationsList, 5000);
			const children = await list.$$('XCUIElementTypeCell');
			return children.length;
		} catch {
			return 0;
		}
	}

	/**
	 * Get the walker's current rating
	 */
	async getRating(): Promise<string> {
		return this.getText(this.ratingDisplay);
	}

	/**
	 * Get the walker's review count
	 */
	async getReviewsCount(): Promise<string> {
		return this.getText(this.reviewsCount);
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
	 * Check if bio field is displayed
	 */
	async isBioFieldDisplayed(): Promise<boolean> {
		return this.isVisible(this.bioField, 5000);
	}

	/**
	 * Check if experience field is displayed
	 */
	async isExperienceFieldDisplayed(): Promise<boolean> {
		return this.isVisible(this.experienceField, 5000);
	}

	/**
	 * Update profile with bio and experience
	 */
	async updateProfile(bio: string, experience?: string): Promise<void> {
		await this.enterBio(bio);
		if (experience) {
			await this.enterExperience(experience);
		}
		await this.saveProfile();
	}

	/**
	 * Clear the bio field
	 */
	async clearBio(): Promise<void> {
		const element = await $(`~${this.bioField}`);
		await element.clearValue();
	}

	/**
	 * Clear the experience field
	 */
	async clearExperience(): Promise<void> {
		const element = await $(`~${this.experienceField}`);
		await element.clearValue();
	}
}

export const walkerProfilePage = new WalkerProfilePage();
