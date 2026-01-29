/**
 * Test account credentials and data
 * Uses environment variables with fallbacks to default seed data values
 */

export const testAccounts = {
	customer: {
		email: process.env.TEST_CUSTOMER_EMAIL || 'appium-test@test.com',
		password: process.env.TEST_CUSTOMER_PASSWORD || 'TestPassword123!',
		firstName: 'Appium',
		lastName: 'Test',
	},
	walker: {
		email: process.env.TEST_WALKER_EMAIL || 'appium-walker@test.com',
		password: process.env.TEST_WALKER_PASSWORD || 'TestPassword123!',
		firstName: 'Appium',
		lastName: 'Walker',
	},
	multiOrg: {
		email: 'customer@demo.com',
		password: 'TestPassword123!',
		orgs: ['OFFLEASH Demo'],
	},
	invalid: {
		email: 'invalid@offleash.test',
		password: 'wrongpassword',
	},
	demoCustomer: {
		email: 'customer@demo.com',
		password: 'TestPassword123!',
		firstName: 'Demo',
		lastName: 'Customer',
	},
	demoAdmin: {
		email: process.env.TEST_ADMIN_EMAIL || 'admin@demo.com',
		password: process.env.TEST_ADMIN_PASSWORD || 'TestPassword123!',
		firstName: 'Demo',
		lastName: 'Admin',
	},
};

export const testPet = {
	name: 'Test Dog',
	breed: 'Labrador Retriever',
	age: '3',
	weight: '65',
	notes: 'Friendly and loves walks',
};

export const testLocation = {
	name: 'Home',
	address: '1234 Test Street',
	city: 'Denver',
	state: 'CO',
	zip: '80202',
};

export const testBooking = {
	notes: 'Please ring doorbell twice',
	recurringFrequency: 'weekly',
	recurringCount: 4,
};

export const testSubscription = {
	frequency: 'weekly',
	dayOfWeek: 'Monday',
	timeSlot: '09:00',
	serviceName: 'Dog Walking',
};

export const testOrganization = {
	name: 'Test Organization',
	inviteCode: 'TEST-INVITE-CODE',
};

export const testPaymentCard = {
	number: '4242424242424242',
	expiry: '12/28',
	cvc: '123',
	zip: '94102',
};

/**
 * Generate unique email for registration tests
 */
export function generateUniqueEmail(): string {
	const timestamp = Date.now();
	const random = Math.random().toString(36).substring(7);
	return 'test-' + timestamp + '-' + random + '@offleash.test';
}

/**
 * Generate unique org name
 */
export function generateUniqueOrgName(): string {
	return 'Test Org ' + Date.now();
}

/**
 * Generate weak password for validation tests
 */
export const weakPasswords = ['123456', 'password', 'abc', 'test'];

/**
 * Generate strong password
 */
export const strongPassword = 'SecureP@ss123!';
