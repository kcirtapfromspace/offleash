import { Page, APIRequestContext, expect } from '@playwright/test';
import { TEST_USERS, API_ENDPOINTS } from './constants';

export interface AuthResponse {
  token: string;
  user: {
    id: string;
    email: string;
    first_name: string;
    last_name: string;
    role: string;
  };
  membership?: {
    id: string;
    organization_id: string;
    organization_name: string;
    organization_slug: string;
    role: string;
    is_default: boolean;
  };
  memberships?: Array<{
    id: string;
    organization_id: string;
    organization_name: string;
    organization_slug: string;
    role: string;
    is_default: boolean;
  }>;
}

export interface SwitchContextResponse {
  token: string;
  membership: {
    id: string;
    organization_id: string;
    organization_name: string;
    organization_slug: string;
    role: string;
  };
}

/**
 * Login via API and return auth response
 */
export async function loginViaAPI(
  request: APIRequestContext,
  email: string,
  password: string
): Promise<AuthResponse> {
  const response = await request.post(API_ENDPOINTS.LOGIN, {
    data: { email, password },
  });

  expect(response.ok()).toBeTruthy();
  return response.json();
}

/**
 * Switch organization context via API
 */
export async function switchContext(
  request: APIRequestContext,
  token: string,
  membershipId: string
): Promise<SwitchContextResponse> {
  const response = await request.post(API_ENDPOINTS.SWITCH_CONTEXT, {
    headers: { Authorization: `Bearer ${token}` },
    data: { membership_id: membershipId },
  });

  expect(response.ok()).toBeTruthy();
  return response.json();
}

/**
 * Get session info via API
 */
export async function getSession(
  request: APIRequestContext,
  token: string
): Promise<AuthResponse> {
  const response = await request.get(API_ENDPOINTS.SESSION, {
    headers: { Authorization: `Bearer ${token}` },
  });

  expect(response.ok()).toBeTruthy();
  return response.json();
}

/**
 * Login as customer via UI
 */
export async function loginAsCustomerUI(page: Page): Promise<void> {
  await page.goto('/login');
  await page.fill('input[name="email"]', TEST_USERS.customer.email);
  await page.fill('input[name="password"]', TEST_USERS.customer.password);
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/(services|bookings|dashboard)/);
}

/**
 * Login as admin via UI
 */
export async function loginAsAdminUI(page: Page): Promise<void> {
  await page.goto('/login');
  await page.fill('input[name="email"]', TEST_USERS.admin.email);
  await page.fill('input[name="password"]', TEST_USERS.admin.password);
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/dashboard/);
}

/**
 * Login as walker via UI
 */
export async function loginAsWalkerUI(page: Page): Promise<void> {
  await page.goto('/login');
  await page.fill('input[name="email"]', TEST_USERS.walker.email);
  await page.fill('input[name="password"]', TEST_USERS.walker.password);
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/dashboard/);
}

/**
 * Login as platform admin via UI
 */
export async function loginAsPlatformAdminUI(page: Page): Promise<void> {
  await page.goto('/login');
  await page.fill('input[name="email"]', TEST_USERS.platformAdmin.email);
  await page.fill('input[name="password"]', TEST_USERS.platformAdmin.password);
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/dashboard/);
}

/**
 * Logout via UI
 */
export async function logoutUI(page: Page): Promise<void> {
  await page.goto('/logout');
  await page.waitForURL(/\/login/);
}

/**
 * Helper to get auth header
 */
export function authHeader(token: string): { Authorization: string } {
  return { Authorization: `Bearer ${token}` };
}

/**
 * Login as customer via API and return token
 */
export async function loginAsCustomer(request: APIRequestContext): Promise<string> {
  const response = await loginViaAPI(
    request,
    TEST_USERS.customer.email,
    TEST_USERS.customer.password
  );

  // Switch context if needed
  if (response.memberships && response.memberships.length > 0) {
    const defaultMembership = response.memberships.find((m) => m.is_default) || response.memberships[0];
    const switchResponse = await switchContext(request, response.token, defaultMembership.id);
    return switchResponse.token;
  }

  return response.token;
}

/**
 * Login as walker via API and return token
 */
export async function loginAsWalker(request: APIRequestContext): Promise<string> {
  const response = await loginViaAPI(
    request,
    TEST_USERS.walker.email,
    TEST_USERS.walker.password
  );

  if (response.memberships && response.memberships.length > 0) {
    const walkerMembership = response.memberships.find((m) => m.role === 'walker') || response.memberships[0];
    const switchResponse = await switchContext(request, response.token, walkerMembership.id);
    return switchResponse.token;
  }

  return response.token;
}

/**
 * Login as admin via API and return token
 */
export async function loginAsAdmin(request: APIRequestContext): Promise<string> {
  const response = await loginViaAPI(
    request,
    TEST_USERS.admin.email,
    TEST_USERS.admin.password
  );

  if (response.memberships && response.memberships.length > 0) {
    const adminMembership = response.memberships.find((m) => m.role === 'admin' || m.role === 'owner') || response.memberships[0];
    const switchResponse = await switchContext(request, response.token, adminMembership.id);
    return switchResponse.token;
  }

  return response.token;
}

/**
 * Login as owner via API and return token
 */
export async function loginAsOwner(request: APIRequestContext): Promise<string> {
  const response = await loginViaAPI(
    request,
    TEST_USERS.owner.email,
    TEST_USERS.owner.password
  );

  if (response.memberships && response.memberships.length > 0) {
    const ownerMembership = response.memberships.find((m) => m.role === 'owner') || response.memberships[0];
    const switchResponse = await switchContext(request, response.token, ownerMembership.id);
    return switchResponse.token;
  }

  return response.token;
}

/**
 * Login as platform admin via API and return token
 */
export async function loginAsPlatformAdmin(request: APIRequestContext): Promise<string> {
  const response = await request.post(API_ENDPOINTS.PLATFORM_LOGIN, {
    data: {
      email: TEST_USERS.platformAdmin.email,
      password: TEST_USERS.platformAdmin.password,
    },
  });

  expect(response.ok()).toBeTruthy();
  const data = await response.json();
  return data.token;
}
