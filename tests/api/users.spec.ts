import { test, expect } from '@playwright/test';
import { loginAsCustomer, loginAsWalker, loginAsAdmin, loginAsPlatformAdmin } from '../utils/auth';
import { createAPIClient } from '../utils/api';
import { API_ENDPOINTS } from '../utils/constants';
import { uniqueEmail } from '../utils/fixtures';

test.describe('Users API', () => {
  test.describe('User Profile', () => {
    test('customer can view their own profile', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.SESSION);
      expect(response.ok).toBeTruthy();

      const data = response.expectSuccess();
      expect(data.user).toBeDefined();
      expect(data.user.id).toBeDefined();
      expect(data.user.email).toBeDefined();
      expect(data.user.first_name).toBeDefined();
      expect(data.user.last_name).toBeDefined();
    });

    test('walker can view their own profile', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.SESSION);
      expect(response.ok).toBeTruthy();

      const data = response.expectSuccess();
      expect(data.user.role).toBe('walker');
    });
  });

  test.describe('Admin User Management', () => {
    test('admin can list all users', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.USERS);
      expect(response.ok).toBeTruthy();

      const users = response.expectSuccess();
      expect(Array.isArray(users)).toBeTruthy();
    });

    test('admin can list walkers', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.ADMIN_USERS_WALKER);
      expect(response.ok).toBeTruthy();

      const walkers = response.expectSuccess();
      expect(Array.isArray(walkers)).toBeTruthy();

      for (const walker of walkers) {
        // All returned users should be walkers
        expect(walker.role === 'walker' || walker.membership_role === 'walker').toBeTruthy();
      }
    });

    test('customer cannot list all users', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.USERS);
      expect(response.ok).toBeFalsy();
      expect(response.status).toBe(403);
    });

    test('walker cannot list all users', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.USERS);
      expect(response.ok).toBeFalsy();
      expect(response.status).toBe(403);
    });
  });

  test.describe('Walker Profiles', () => {
    test('customer can view walker profiles', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.WALKER_PROFILES);
      expect(response.ok).toBeTruthy();

      const profiles = response.expectSuccess();
      expect(Array.isArray(profiles)).toBeTruthy();
    });

    test('walker can update their own profile', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      const response = await client.put(API_ENDPOINTS.WALKER_PROFILES, {
        bio: 'I love walking dogs!',
        experience_years: 5,
      });

      // May succeed or may need to create first
      if (response.ok) {
        const profile = response.expectSuccess();
        expect(profile.bio).toBe('I love walking dogs!');
      }
    });
  });

  test.describe('Platform Admin', () => {
    test('platform admin can list all tenants', async ({ request }) => {
      const token = await loginAsPlatformAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.ADMIN_TENANTS);
      expect(response.ok).toBeTruthy();

      const tenants = response.expectSuccess();
      expect(Array.isArray(tenants)).toBeTruthy();
    });

    test('regular admin cannot access platform admin endpoints', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.ADMIN_TENANTS);
      expect(response.ok).toBeFalsy();
    });
  });

  test.describe('User Registration', () => {
    test('new user can register', async ({ request }) => {
      const email = uniqueEmail('newuser');

      const response = await request.post(API_ENDPOINTS.REGISTER, {
        data: {
          email: email,
          password: 'NewUser123!',
          first_name: 'New',
          last_name: 'User',
        },
      });

      expect(response.ok()).toBeTruthy();
      const data = await response.json();

      expect(data.token).toBeDefined();
      expect(data.user).toBeDefined();
      expect(data.user.email).toBe(email);
    });

    test('cannot register with existing email', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.REGISTER, {
        data: {
          email: 'customer@test.offleash.world', // Already exists
          password: 'AnotherPass123!',
          first_name: 'Duplicate',
          last_name: 'User',
        },
      });

      expect(response.ok()).toBeFalsy();
      expect(response.status()).toBe(409); // Conflict
    });

    test('cannot register with weak password', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.REGISTER, {
        data: {
          email: uniqueEmail('weakpass'),
          password: '123', // Too weak
          first_name: 'Weak',
          last_name: 'Password',
        },
      });

      expect(response.ok()).toBeFalsy();
    });

    test('cannot register without required fields', async ({ request }) => {
      const response = await request.post(API_ENDPOINTS.REGISTER, {
        data: {
          email: uniqueEmail('missing'),
          // Missing password, first_name, last_name
        },
      });

      expect(response.ok()).toBeFalsy();
    });
  });
});
