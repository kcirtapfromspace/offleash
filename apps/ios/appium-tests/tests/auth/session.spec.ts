import { RoleSelectionPage } from '../../pages/auth/RoleSelectionPage.js';
import { LoginPage } from '../../pages/auth/LoginPage.js';
import {
	expectAnyVisible,
	expectVisible,
	logout,
	relaunchApp,
	resetApp,
} from '../../utils/helpers.js';
import { ids } from '../../utils/ids.js';
import { testAccounts } from '../../utils/test-data.js';
import { expireSession } from '../../utils/api-helpers.js';

describe('Auth - Session', () => {
	const isMockAuth = process.env.OFFLEASH_TEST_AUTH === 'mock';

	beforeEach(async () => {
		await resetApp();
	});

	it('AUTH-008 Session Persistence', async () => {
		await RoleSelectionPage.selectCustomer();
		await LoginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await expectAnyVisible([
			ids.customer.servicesList,
			ids.customer.loadingIndicator,
			ids.customer.emptyState,
		]);
		await relaunchApp();
		await expectAnyVisible([
			ids.customer.servicesList,
			ids.customer.loadingIndicator,
			ids.customer.emptyState,
		]);
	});

	it('AUTH-009 Session Expiration', async () => {
		await RoleSelectionPage.selectCustomer();
		await LoginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await expectAnyVisible([
			ids.customer.servicesList,
			ids.customer.loadingIndicator,
			ids.customer.emptyState,
		]);
		if (isMockAuth) {
			await logout();
			await relaunchApp();
		} else {
			await expireSession();
			await relaunchApp();
		}
		await expectVisible(ids.auth.roleCustomer);
	});

	it('AUTH-010 Logout', async () => {
		await RoleSelectionPage.selectCustomer();
		await LoginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await expectAnyVisible([
			ids.customer.servicesList,
			ids.customer.loadingIndicator,
			ids.customer.emptyState,
		]);
		await logout();
	});
});
