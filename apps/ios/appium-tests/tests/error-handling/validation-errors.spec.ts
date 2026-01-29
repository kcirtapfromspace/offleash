import { RoleSelectionPage } from '../../pages/auth/RoleSelectionPage.js';
import { LoginPage } from '../../pages/auth/LoginPage.js';
import { expectVisible, resetApp, tap, typeText } from '../../utils/helpers.js';
import { ids } from '../../utils/ids.js';

describe('Error Handling - Validation', () => {
	beforeEach(async () => {
		await resetApp();
	});

	it('ERR-005 Validation Error', async () => {
		await RoleSelectionPage.selectCustomer();
		await LoginPage.expectVisible();
		await typeText(ids.auth.loginEmail, 'bad-email');
		await typeText(ids.auth.loginPassword, 'short');
		await tap(ids.auth.loginSubmit);
		await expectVisible(ids.errors.validationError);
	});
});
