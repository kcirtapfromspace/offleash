import { RoleSelectionPage } from '../../pages/auth/RoleSelectionPage.js';
import { LoginPage } from '../../pages/auth/LoginPage.js';
import { expectPresent, resetApp } from '../../utils/helpers.js';
import { ids } from '../../utils/ids.js';

describe('Auth - OAuth', () => {
	beforeEach(async () => {
		await resetApp();
	});

	it('AUTH-007 Google OAuth', async () => {
		await RoleSelectionPage.selectCustomer();
		await LoginPage.loginWithGoogle();
		await expectPresent(ids.auth.errorBanner);
	});
});
