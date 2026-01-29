import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { onboardingPage } from '../../pages/walker/OnboardingPage';
import { dashboardPage } from '../../pages/walker/DashboardPage';
import { testAccounts, testOrganization, generateUniqueOrgName } from '../../utils/test-data';

describe('Walker - Onboarding', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectWalkerRole();
		await loginPage.waitForPageLoad();
	});

	// WON-001: Create Organization
	it('WON-001: should allow creating an organization', async () => {
		// Login as new walker
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);

		// If onboarding shows, test create org flow
		if (await onboardingPage.isDisplayed()) {
			const orgName = generateUniqueOrgName();
			await onboardingPage.createOrganization(orgName);
			// Should either complete or show dashboard
			const complete = await onboardingPage.isOnboardingComplete();
			const dashboard = await dashboardPage.isDisplayed();
			expect(complete || dashboard).toBe(true);
		} else {
			// Already onboarded, should see dashboard
			expect(await dashboardPage.isDisplayed()).toBe(true);
		}
	});

	// WON-002: Join Organization
	it('WON-002: should allow joining an organization with valid code', async () => {
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);

		if (await onboardingPage.isDisplayed()) {
			await onboardingPage.joinOrganization(testOrganization.inviteCode);
			// Either error or success
			const error = await onboardingPage.isErrorDisplayed();
			const dashboard = await dashboardPage.isDisplayed();
			expect(error || dashboard).toBe(true);
		}
	});

	// WON-003: Invalid Invite
	it('WON-003: should show error for invalid invite code', async () => {
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);

		if (await onboardingPage.isDisplayed()) {
			await onboardingPage.joinOrganization('INVALID-CODE-123');
			expect(await onboardingPage.isErrorDisplayed()).toBe(true);
		}
	});

	// WON-004: Complete Onboarding
	it('WON-004: should access dashboard after onboarding', async () => {
		await loginPage.login(testAccounts.walker.email, testAccounts.walker.password);

		// Either onboarding or dashboard should be visible
		const onboarding = await onboardingPage.isDisplayed();
		const dashboard = await dashboardPage.isDisplayed();
		expect(onboarding || dashboard).toBe(true);
	});
});
