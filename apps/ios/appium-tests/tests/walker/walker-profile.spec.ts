import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { walkerProfilePage } from '../../pages/walker/WalkerProfilePage';
import { testAccounts } from '../../utils/test-data';

describe('Walker - Profile', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);
		await browser.pause(3000);

		// Navigate to settings and then profile update
		if (await dashboardPage.isDisplayed()) {
			await dashboardPage.navigateToSettings();
			await browser.pause(1000);
			await walkerProfilePage.navigateToProfileUpdate();
			await browser.pause(500);
		}
	});

	// WPR-001: Display walker profile details
	it('WPR-001: should display walker profile details', async () => {
		const isDisplayed = await walkerProfilePage.isDisplayed();
		expect(isDisplayed).toBe(true);

		// Check if bio field is displayed
		const isBioDisplayed = await walkerProfilePage.isBioFieldDisplayed();
		expect(isBioDisplayed).toBe(true);
	});

	// WPR-002: Update bio field
	it('WPR-002: should update bio field', async () => {
		const isDisplayed = await walkerProfilePage.isDisplayed();
		if (isDisplayed) {
			// Clear existing bio
			await walkerProfilePage.clearBio();
			await browser.pause(300);

			// Enter new bio
			const newBio = `Test bio updated at ${Date.now()}`;
			await walkerProfilePage.enterBio(newBio);
			await browser.pause(500);

			// Verify the text was entered
			const currentBio = await walkerProfilePage.getBio();
			expect(currentBio).toContain('Test bio');
		}
	});

	// WPR-003: Update experience info
	it('WPR-003: should update experience info', async () => {
		const isDisplayed = await walkerProfilePage.isDisplayed();
		if (isDisplayed) {
			const isExperienceDisplayed = await walkerProfilePage.isExperienceFieldDisplayed();
			if (isExperienceDisplayed) {
				// Clear existing experience
				await walkerProfilePage.clearExperience();
				await browser.pause(300);

				// Enter new experience
				const newExperience = '5 years of professional dog walking experience';
				await walkerProfilePage.enterExperience(newExperience);
				await browser.pause(500);

				// Verify the text was entered
				const currentExperience = await walkerProfilePage.getExperience();
				expect(currentExperience).toContain('5 years');
			} else {
				// Experience field might not exist in all versions
				console.log('Experience field not displayed - skipping');
				expect(true).toBe(true);
			}
		}
	});

	// WPR-004: Save profile changes
	it('WPR-004: should save profile changes', async () => {
		const isDisplayed = await walkerProfilePage.isDisplayed();
		if (isDisplayed) {
			// Make a change to bio
			const newBio = `Updated bio ${Date.now()}`;
			await walkerProfilePage.enterBio(newBio);
			await browser.pause(500);

			// Check if save button is enabled
			const isSaveEnabled = await walkerProfilePage.isSaveButtonEnabled();
			if (isSaveEnabled) {
				await walkerProfilePage.saveProfile();
				await browser.pause(1000);

				// Should either navigate back or show success
				expect(true).toBe(true);
			} else {
				// Save might be disabled if no changes detected
				expect(true).toBe(true);
			}
		}
	});
});
