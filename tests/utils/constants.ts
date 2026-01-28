/**
 * Test user credentials
 */
export const TEST_USERS = {
  customer: {
    email: 'customer@test.offleash.world',
    password: 'TestPassword123!',
    firstName: 'Test',
    lastName: 'Customer',
  },
  walker: {
    email: 'walker@test.offleash.world',
    password: 'TestPassword123!',
    firstName: 'Test',
    lastName: 'Walker',
  },
  admin: {
    email: 'admin@test.offleash.world',
    password: 'TestPassword123!',
    firstName: 'Test',
    lastName: 'Admin',
  },
  owner: {
    email: 'owner@test.offleash.world',
    password: 'TestPassword123!',
    firstName: 'Test',
    lastName: 'Owner',
  },
  platformAdmin: {
    email: 'platform@test.offleash.world',
    password: 'TestPassword123!',
    firstName: 'Platform',
    lastName: 'Admin',
  },
  // Secondary org user for isolation tests
  orgBCustomer: {
    email: 'customer@org-b.test',
    password: 'TestPassword123!',
    firstName: 'OrgB',
    lastName: 'Customer',
  },
};

/**
 * Test organization data
 */
export const TEST_ORGS = {
  demo: {
    id: '00000000-0000-0000-0000-000000000001',
    name: 'OFFLEASH Demo',
    slug: 'offleash-demo',
  },
  orgB: {
    id: '00000000-0000-0000-0000-000000000002',
    name: 'Test Org B',
    slug: 'test-org-b',
  },
};

/**
 * API endpoints
 */
export const API_ENDPOINTS = {
  // Auth
  LOGIN: '/auth/login/universal',
  REGISTER: '/auth/register',
  SESSION: '/auth/session',
  PLATFORM_LOGIN: '/platform/auth/login',

  // Context
  SWITCH_CONTEXT: '/contexts/switch',
  LIST_CONTEXTS: '/contexts',
  CLEAR_CONTEXT: '/contexts/clear',

  // Bookings
  BOOKINGS: '/bookings',
  BOOKINGS_CUSTOMER: '/bookings/customer',
  BOOKINGS_WALKER: '/bookings/walker',
  RECURRING_BOOKINGS: '/bookings/recurring',

  // Services
  SERVICES: '/services',

  // Availability (walker-specific)
  AVAILABILITY: '/availability', // needs /:walker_id appended
  AVAILABILITY_SLOTS: '/availability/slots',

  // Calendar
  CALENDAR_EVENTS: '/calendar/events',

  // Users
  USERS: '/users',
  USERS_ME: '/users/me',
  ADMIN_WALKERS: '/admin/walkers',

  // Working Hours (needs /:walker_id appended)
  WORKING_HOURS: '/working-hours',

  // Walker Profiles
  WALKER_PROFILE: '/walker/profile',
  ADMIN_WALKER_PROFILE: '/admin/walkers', // needs /:walker_id/profile appended

  // Locations
  LOCATIONS: '/locations',

  // Payment
  PAYMENT_METHODS: '/payment-methods',
  CHECKOUT: '/checkout',

  // Admin
  ADMIN_TENANTS: '/admin/tenants',

  // Health
  HEALTH: '/health',
};

/**
 * Test service data
 */
export const TEST_SERVICES = {
  walk30: {
    name: '30 Minute Walk',
    description: 'A quick 30-minute walk for your pup',
    duration_minutes: 30,
    price_cents: 2500,
  },
  walk60: {
    name: '60 Minute Walk',
    description: 'A full hour walk with playtime',
    duration_minutes: 60,
    price_cents: 4000,
  },
  petSitting: {
    name: 'Pet Sitting',
    description: 'In-home pet sitting for extended periods',
    duration_minutes: 120,
    price_cents: 5000,
  },
};

/**
 * Test location data
 */
export const TEST_LOCATIONS = {
  home: {
    address: '123 Test Street',
    city: 'Denver',
    state: 'CO',
    zip: '80202',
    latitude: 39.7392,
    longitude: -104.9903,
    notes: 'Ring doorbell, dog is friendly',
  },
  work: {
    address: '456 Work Avenue',
    city: 'Denver',
    state: 'CO',
    zip: '80203',
    latitude: 39.7312,
    longitude: -104.9826,
    notes: 'Meet in lobby',
  },
};

/**
 * Booking status values
 */
export const BOOKING_STATUS = {
  PENDING: 'pending',
  CONFIRMED: 'confirmed',
  IN_PROGRESS: 'in_progress',
  COMPLETED: 'completed',
  CANCELLED: 'cancelled',
  NO_SHOW: 'no_show',
};

/**
 * Recurrence frequency values
 */
export const RECURRENCE_FREQUENCY = {
  WEEKLY: 'weekly',
  BI_WEEKLY: 'bi_weekly',
  MONTHLY: 'monthly',
};

/**
 * Timeouts for various operations
 */
export const TIMEOUTS = {
  SHORT: 5000,
  MEDIUM: 10000,
  LONG: 30000,
  VERY_LONG: 60000,
};
