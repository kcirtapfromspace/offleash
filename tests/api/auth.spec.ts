import { test, expect } from '@playwright/test';
import { TEST_USERS, API_ENDPOINTS, TEST_ORGS } from '../utils/constants';
import { loginViaAPI, switchContext, getSession } from '../utils/auth';
import { makeRequest } from '../utils/api';

test.describe('Authentication API', () => {
  test.describe('Login', () => {
    test('should login customer successfully', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          email: TEST_USERS.customer.email,
          password: TEST_USERS.customer.password,
        },
      });

      expect(response.ok()).toBeTruthy();
      const data = await response.json();

      expect(data.token).toBeDefined();
      expect(data.user).toBeDefined();
      expect(data.user.email).toBe(TEST_USERS.customer.email);
      expect(data.user.first_name).toBe(TEST_USERS.customer.firstName);
      expect(data.user.last_name).toBe(TEST_USERS.customer.lastName);
    });

    test('should login walker successfully', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          email: TEST_USERS.walker.email,
          password: TEST_USERS.walker.password,
        },
      });

      expect(response.ok()).toBeTruthy();
      const data = await response.json();

      expect(data.token).toBeDefined();
      expect(data.user.email).toBe(TEST_USERS.walker.email);
    });

    test('should login admin successfully', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          email: TEST_USERS.admin.email,
          password: TEST_USERS.admin.password,
        },
      });

      expect(response.ok()).toBeTruthy();
      const data = await response.json();

      expect(data.token).toBeDefined();
      expect(data.user.email).toBe(TEST_USERS.admin.email);
    });

    test('should login owner successfully', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          email: TEST_USERS.owner.email,
          password: TEST_USERS.owner.password,
        },
      });

      expect(response.ok()).toBeTruthy();
      const data = await response.json();

      expect(data.token).toBeDefined();
      expect(data.user.email).toBe(TEST_USERS.owner.email);
    });

    test('should fail login with wrong password', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          email: TEST_USERS.customer.email,
          password: 'WrongPassword123!',
        },
      });

      expect(response.status()).toBe(401);
    });

    test('should fail login with non-existent email', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          email: 'nonexistent@test.offleash.world',
          password: TEST_USERS.customer.password,
        },
      });

      expect(response.status()).toBe(401);
    });

    test('should fail login with missing email', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          password: TEST_USERS.customer.password,
        },
      });

      expect(response.ok()).toBeFalsy();
    });

    test('should fail login with missing password', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.LOGIN, {
        data: {
          email: TEST_USERS.customer.email,
        },
      });

      expect(response.ok()).toBeFalsy();
    });
  });

  test.describe('Platform Admin Login', () => {
    test('should login platform admin successfully', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.PLATFORM_LOGIN, {
        data: {
          email: TEST_USERS.platformAdmin.email,
          password: TEST_USERS.platformAdmin.password,
        },
      });

      expect(response.ok()).toBeTruthy();
      const data = await response.json();

      expect(data.token).toBeDefined();
    });

    test('should fail platform login with regular user credentials', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.PLATFORM_LOGIN, {
        data: {
          email: TEST_USERS.customer.email,
          password: TEST_USERS.customer.password,
        },
      });

      expect(response.ok()).toBeFalsy();
    });
  });

  test.describe('Session', () => {
    test('should get session with valid token', async ({ request }) => {
      const authResponse = await loginViaAPI(
        request,
        TEST_USERS.customer.email,
        TEST_USERS.customer.password
      );

      const sessionResponse = await request.get(API_ENDPOINTS.SESSION, {
        headers: { Authorization: `Bearer ${authResponse.token}` },
      });

      expect(sessionResponse.ok()).toBeTruthy();
      const data = await sessionResponse.json();

      expect(data.user).toBeDefined();
      expect(data.user.email).toBe(TEST_USERS.customer.email);
    });

    test('should fail to get session without token', async ({ request }) => {
      const response = await request.get(API_ENDPOINTS.SESSION);

      expect(response.status()).toBe(401);
    });

    test('should fail to get session with invalid token', async ({ request }) => {
      const response = await request.get(API_ENDPOINTS.SESSION, {
        headers: { Authorization: 'Bearer invalid-token-here' },
      });

      expect(response.status()).toBe(401);
    });
  });

  test.describe('Context Switching', () => {
    test('should list available contexts (memberships)', async ({ request }) => {
      const authResponse = await loginViaAPI(
        request,
        TEST_USERS.customer.email,
        TEST_USERS.customer.password
      );

      expect(authResponse.memberships).toBeDefined();
      expect(authResponse.memberships!.length).toBeGreaterThan(0);

      const membership = authResponse.memberships![0];
      expect(membership.organization_id).toBeDefined();
      expect(membership.role).toBe('customer');
    });

    test('should switch to a different context', async ({ request }) => {
      const authResponse = await loginViaAPI(
        request,
        TEST_USERS.customer.email,
        TEST_USERS.customer.password
      );

      const membershipId = authResponse.memberships![0].id;
      const switchResponse = await switchContext(request, authResponse.token, membershipId);

      expect(switchResponse.token).toBeDefined();
      expect(switchResponse.token).not.toBe(authResponse.token); // Should be a new token
      expect(switchResponse.membership).toBeDefined();
      expect(switchResponse.membership.id).toBe(membershipId);
    });

    test('should fail to switch to invalid membership', async ({ request }) => {
      const authResponse = await loginViaAPI(
        request,
        TEST_USERS.customer.email,
        TEST_USERS.customer.password
      );

      const response = await request.post(API_ENDPOINTS.SWITCH_CONTEXT, {
        headers: { Authorization: `Bearer ${authResponse.token}` },
        data: { membership_id: '00000000-0000-0000-0000-000000000000' },
      });

      expect(response.ok()).toBeFalsy();
    });
  });

  test.describe('Multi-tenant Isolation', () => {
    test('customer from Org A cannot access Org B resources', async ({ request }) => {
      // Login as Org A customer and switch to their context
      const orgAAuth = await loginViaAPI(
        request,
        TEST_USERS.customer.email,
        TEST_USERS.customer.password
      );
      const orgAMembership = orgAAuth.memberships!.find(
        m => m.organization_id === TEST_ORGS.demo.id
      );
      expect(orgAMembership).toBeDefined();
      const orgAContext = await switchContext(request, orgAAuth.token, orgAMembership!.id);

      // Login as Org B customer
      const orgBAuth = await loginViaAPI(
        request,
        TEST_USERS.orgBCustomer.email,
        TEST_USERS.orgBCustomer.password
      );
      const orgBMembership = orgBAuth.memberships!.find(
        m => m.organization_id === TEST_ORGS.orgB.id
      );
      expect(orgBMembership).toBeDefined();
      const orgBContext = await switchContext(request, orgBAuth.token, orgBMembership!.id);

      // Org A customer fetches services
      const orgAServicesRes = await request.get(API_ENDPOINTS.SERVICES, {
        headers: { Authorization: `Bearer ${orgAContext.token}` },
      });
      expect(orgAServicesRes.ok()).toBeTruthy();
      const orgAServices = await orgAServicesRes.json();

      // Org B customer fetches services
      const orgBServicesRes = await request.get(API_ENDPOINTS.SERVICES, {
        headers: { Authorization: `Bearer ${orgBContext.token}` },
      });
      expect(orgBServicesRes.ok()).toBeTruthy();
      const orgBServices = await orgBServicesRes.json();

      // Services should be different between orgs
      const orgAServiceIds = orgAServices.map((s: any) => s.id);
      const orgBServiceIds = orgBServices.map((s: any) => s.id);

      // Verify no overlap in service IDs (each org has its own services)
      const overlap = orgAServiceIds.filter((id: string) => orgBServiceIds.includes(id));
      expect(overlap.length).toBe(0);
    });
  });
});
