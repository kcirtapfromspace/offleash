import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { registerPage } from '../../pages/auth/RegisterPage';
import { generateUniqueEmail, weakPasswords, strongPassword } from '../../utils/test-data';

describe('Authentication - Registration', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.tapRegister();
		await registerPage.waitForPageLoad();
	});

	// AUTH-006: Registration Flow
	it('AUTH-006: should complete registration flow', async () => {
		const email = generateUniqueEmail();
		await registerPage.register('Test', 'User', email, strongPassword);
		expect(await registerPage.isVerifyBannerDisplayed()).toBe(true);
	});

	it('should show error for weak password', async () => {
		const email = generateUniqueEmail();
		await registerPage.enterFirstName('Test');
		await registerPage.enterLastName('User');
		await registerPage.enterEmail(email);
		await registerPage.enterPassword(weakPasswords[0]);
		expect(await registerPage.isPasswordStrengthDisplayed()).toBe(true);
	});

	it('should show error for invalid email format', async () => {
		await registerPage.enterFirstName('Test');
		await registerPage.enterLastName('User');
		await registerPage.enterEmail('invalid-email');
		await registerPage.enterPassword(strongPassword);
		await registerPage.tapSubmit();
		expect(await registerPage.isErrorDisplayed()).toBe(true);
	});

	it('should require all fields', async () => {
		await registerPage.tapSubmit();
		expect(await registerPage.isErrorDisplayed()).toBe(true);
	});
});
