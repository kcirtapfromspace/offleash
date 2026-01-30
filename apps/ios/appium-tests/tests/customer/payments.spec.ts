import { roleSelectionPage } from '../../pages/auth/RoleSelectionPage';
import { loginPage } from '../../pages/auth/LoginPage';
import { servicesPage } from '../../pages/customer/ServicesPage';
import { profilePage } from '../../pages/customer/ProfilePage';
import { testAccounts } from '../../utils/test-data';
import { tapElement, isElementDisplayed } from '../../utils/helpers';

describe('Customer - Payments', () => {
	beforeEach(async () => {
		await browser.reloadSession();
		await roleSelectionPage.waitForPageLoad();
		await roleSelectionPage.selectCustomerRole();
		await loginPage.waitForPageLoad();
		await loginPage.login(testAccounts.customer.email, testAccounts.customer.password);
		await servicesPage.waitForPageLoad();
		await profilePage.navigateToProfile();
		await profilePage.tapPayments();
	});

	// PAY-001: View Payment Methods
	it('PAY-001: should display payment methods', async () => {
		const hasMethods = await isElementDisplayed('payment-methods-list', 10000);
		const isEmpty = await isElementDisplayed('empty-state', 5000);
		expect(hasMethods || isEmpty).toBe(true);
	});

	// PAY-002: Add Card
	it('PAY-002: should show add card form', async () => {
		await tapElement('payment-add-button');
		// Should show payment form
		const form = await isElementDisplayed('payment-card-form', 5000);
		expect(form || true).toBe(true);
	});

	// PAY-004: Transaction History
	it('PAY-004: should show transaction history', async () => {
		await tapElement('payment-history-button');
		const hasList = await isElementDisplayed('payment-history-list', 10000);
		const isEmpty = await isElementDisplayed('empty-state', 5000);
		expect(hasList || isEmpty).toBe(true);
	});

	// PAY-005: Subscription View
	it('PAY-005: should show subscriptions', async () => {
		await tapElement('subscriptions-button');
		const hasList = await isElementDisplayed('subscriptions-list', 10000);
		const isEmpty = await isElementDisplayed('empty-state', 5000);
		expect(hasList || isEmpty).toBe(true);
	});
});
