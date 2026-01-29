/**
 * Test account credentials and data
 */

export const testAccounts = {
	customer: {
		email: 'test-customer@offleash.test',
		password: 'TestPass123!',
		firstName: 'Test',
		lastName: 'Customer',
	},
	walker: {
		email: 'test-walker@offleash.test',
		password: 'TestPass123!',
		firstName: 'Test',
		lastName: 'Walker',
	},
	multiOrg: {
		email: 'test-multi@offleash.test',
		password: 'TestPass123!',
		orgs: ['Org A', 'Org B'],
	},
	invalid: {
		email: 'invalid@offleash.test',
		password: 'wrongpassword',
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
	address: '123 Test Street',
	city: 'San Francisco',
	state: 'CA',
	zip: '94102',
};

export const testBooking = {
	notes: 'Please ring doorbell twice',
	recurringFrequency: 'weekly',
	recurringCount: 4,
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
