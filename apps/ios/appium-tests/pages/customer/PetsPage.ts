import { BasePage } from '../BasePage';

/**
 * Page object for the Pets management screen
 * Handles pet CRUD operations (Create, Read, Update, Delete)
 */
export class PetsPage extends BasePage {
	// Selectors from Swift views
	private petsList = 'pets-list';
	private petAddButton = 'pet-add-button';
	private petNameField = 'pet-name-field';
	private petBreedField = 'pet-breed-field';
	private petAgeField = 'pet-age-field';
	private petWeightField = 'pet-weight-field';
	private petNotesField = 'pet-notes-field';
	private petSaveButton = 'pet-save-button';
	private petDeleteButton = 'pet-delete-button';
	private petDeleteConfirm = 'pet-delete-confirm';
	private petEditButton = 'pet-edit-button';
	private petCancelButton = 'pet-cancel-button';
	private emptyState = 'empty-state';

	async waitForPageLoad(): Promise<void> {
		// Either list or empty state should be visible
		try {
			await this.waitFor(this.petsList, 5000);
		} catch {
			await this.waitFor(this.emptyState, 5000);
		}
	}

	async isDisplayed(): Promise<boolean> {
		return (
			(await this.isVisible(this.petsList, 5000)) || (await this.isVisible(this.emptyState, 2000))
		);
	}

	/**
	 * Tap the add pet button to start creating a new pet
	 */
	async tapAddPet(): Promise<void> {
		await this.tap(this.petAddButton);
	}

	/**
	 * Enter the pet's name
	 */
	async enterPetName(name: string): Promise<void> {
		await this.type(this.petNameField, name);
	}

	/**
	 * Enter the pet's breed
	 */
	async enterPetBreed(breed: string): Promise<void> {
		await this.type(this.petBreedField, breed);
	}

	/**
	 * Enter the pet's age
	 */
	async enterPetAge(age: string): Promise<void> {
		await this.type(this.petAgeField, age);
	}

	/**
	 * Enter the pet's weight
	 */
	async enterPetWeight(weight: string): Promise<void> {
		await this.type(this.petWeightField, weight);
	}

	/**
	 * Enter notes about the pet
	 */
	async enterPetNotes(notes: string): Promise<void> {
		await this.type(this.petNotesField, notes);
	}

	/**
	 * Save the pet (create or update)
	 */
	async savePet(): Promise<void> {
		await this.tap(this.petSaveButton);
	}

	/**
	 * Select a pet from the list by name
	 */
	async selectPet(petName: string): Promise<void> {
		await this.tap(`pet-item-${petName}`);
	}

	/**
	 * Tap edit button on selected pet
	 */
	async tapEditPet(): Promise<void> {
		await this.tap(this.petEditButton);
	}

	/**
	 * Tap delete button on selected pet
	 */
	async deletePet(): Promise<void> {
		await this.tap(this.petDeleteButton);
	}

	/**
	 * Confirm pet deletion in the confirmation dialog
	 */
	async confirmDelete(): Promise<void> {
		await this.tap(this.petDeleteConfirm);
	}

	/**
	 * Cancel the add/edit pet flow
	 */
	async cancelAddPet(): Promise<void> {
		await this.tap(this.petCancelButton);
	}

	/**
	 * Get the count of pets in the list
	 */
	async getPetCount(): Promise<number> {
		try {
			const list = await this.waitFor(this.petsList, 5000);
			const children = await list.$$('XCUIElementTypeCell');
			return children.length;
		} catch {
			return 0;
		}
	}

	/**
	 * Check if empty state is displayed (no pets)
	 */
	async isEmptyStateDisplayed(): Promise<boolean> {
		return this.isVisible(this.emptyState, 5000);
	}

	/**
	 * Check if the add pet form is displayed
	 */
	async isAddPetFormDisplayed(): Promise<boolean> {
		return this.isVisible(this.petNameField, 5000);
	}

	/**
	 * Check if save button is enabled
	 */
	async isSaveButtonEnabled(): Promise<boolean> {
		try {
			const element = await $(`~${this.petSaveButton}`);
			return await element.isEnabled();
		} catch {
			return false;
		}
	}

	/**
	 * Add a new pet with all fields
	 */
	async addPetWithDetails(
		name: string,
		breed: string,
		age?: string,
		weight?: string,
		notes?: string
	): Promise<void> {
		await this.tapAddPet();
		await browser.pause(500);
		await this.enterPetName(name);
		await this.enterPetBreed(breed);
		if (age) {
			await this.enterPetAge(age);
		}
		if (weight) {
			await this.enterPetWeight(weight);
		}
		if (notes) {
			await this.enterPetNotes(notes);
		}
		await this.savePet();
	}
}

export const petsPage = new PetsPage();
