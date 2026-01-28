import { test, expect } from '@playwright/test';
import { loginAsCustomer, loginAsAdmin, loginAsWalker } from '../utils/auth';
import { createAPIClient } from '../utils/api';
import { API_ENDPOINTS, TEST_SERVICES } from '../utils/constants';
import { uniqueId } from '../utils/fixtures';

test.describe('Services API', () => {
  test.describe('List Services', () => {
    test('customer can list active services', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get<any[]>(API_ENDPOINTS.SERVICES);
      expect(response.ok).toBeTruthy();

      const services = response.expectSuccess();
      expect(Array.isArray(services)).toBeTruthy();
      expect(services.length).toBeGreaterThan(0);

      // Verify service structure
      const service = services[0];
      expect(service.id).toBeDefined();
      expect(service.name).toBeDefined();
      expect(service.duration_minutes).toBeDefined();
      expect(typeof service.duration_minutes).toBe('number');
      expect(service.price_cents).toBeDefined();
      expect(typeof service.price_cents).toBe('number');
      expect(service.is_active).toBe(true);
    });

    test('walker can list active services', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      const response = await client.get<any[]>(API_ENDPOINTS.SERVICES);
      expect(response.ok).toBeTruthy();

      const services = response.expectSuccess();
      expect(services.length).toBeGreaterThan(0);
    });

    test('admin can list all services', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.get<any[]>(API_ENDPOINTS.SERVICES);
      expect(response.ok).toBeTruthy();

      const services = response.expectSuccess();
      expect(services.length).toBeGreaterThan(0);
    });

    test('unauthenticated request fails', async ({ request }) => {
      const response = await request.get(API_ENDPOINTS.SERVICES);
      expect(response.status()).toBe(401);
    });
  });

  test.describe('Get Service', () => {
    test('customer can get a specific service', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      // First get list of services
      const listResponse = await client.get<any[]>(API_ENDPOINTS.SERVICES);
      const services = listResponse.expectSuccess();
      expect(services.length).toBeGreaterThan(0);

      // Get specific service
      const serviceId = services[0].id;
      const response = await client.get(`${API_ENDPOINTS.SERVICES}/${serviceId}`);
      expect(response.ok).toBeTruthy();

      const service = response.expectSuccess();
      expect(service.id).toBe(serviceId);
    });

    test('returns 404 for non-existent service', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get(`${API_ENDPOINTS.SERVICES}/00000000-0000-0000-0000-000000000000`);
      expect(response.status).toBe(404);
    });
  });

  test.describe('Admin Service Management', () => {
    test('admin can create a service', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const newService = {
        name: `Test Service ${uniqueId()}`,
        description: 'A test service created by API tests',
        duration_minutes: 45,
        base_price_cents: 3500,
      };

      const response = await client.post(API_ENDPOINTS.SERVICES, newService);
      expect(response.ok).toBeTruthy();

      const service = response.expectSuccess();
      expect(service.id).toBeDefined();
      expect(service.name).toBe(newService.name);
      expect(service.duration_minutes).toBe(45);
      expect(service.price_cents).toBe(3500);
      expect(service.is_active).toBe(true);
    });

    test('admin can update a service', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      // Create a service first
      const createResponse = await client.post(API_ENDPOINTS.SERVICES, {
        name: `Update Test ${uniqueId()}`,
        description: 'Original description',
        duration_minutes: 30,
        base_price_cents: 2500,
      });
      const createdService = createResponse.expectSuccess();

      // Update it
      const updateResponse = await client.put(`${API_ENDPOINTS.SERVICES}/${createdService.id}`, {
        name: createdService.name,
        description: 'Updated description',
        duration_minutes: 45,
        base_price_cents: 3000,
      });
      expect(updateResponse.ok).toBeTruthy();

      const updatedService = updateResponse.expectSuccess();
      expect(updatedService.description).toBe('Updated description');
      expect(updatedService.duration_minutes).toBe(45);
      expect(updatedService.base_price_cents).toBe(3000);
    });

    test('admin can deactivate a service', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      // Create a service
      const createResponse = await client.post(API_ENDPOINTS.SERVICES, {
        name: `Deactivate Test ${uniqueId()}`,
        description: 'Will be deactivated',
        duration_minutes: 30,
        base_price_cents: 2500,
      });
      const createdService = createResponse.expectSuccess();

      // Deactivate it
      const updateResponse = await client.put(`${API_ENDPOINTS.SERVICES}/${createdService.id}`, {
        is_active: false,
      });
      expect(updateResponse.ok).toBeTruthy();

      const deactivatedService = updateResponse.expectSuccess();
      expect(deactivatedService.is_active).toBe(false);
    });

    test('customer cannot create a service', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.SERVICES, {
        name: 'Unauthorized Service',
        description: 'Should not be created',
        duration_minutes: 30,
        base_price_cents: 2500,
      });

      expect(response.ok).toBeFalsy();
      expect(response.status).toBe(403);
    });

    test('walker cannot create a service', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.SERVICES, {
        name: 'Unauthorized Service',
        description: 'Should not be created',
        duration_minutes: 30,
        base_price_cents: 2500,
      });

      expect(response.ok).toBeFalsy();
      expect(response.status).toBe(403);
    });
  });

  test.describe('Service Validation', () => {
    test('rejects service with missing name', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.SERVICES, {
        description: 'No name',
        duration_minutes: 30,
        base_price_cents: 2500,
      });

      expect(response.ok).toBeFalsy();
    });

    test('rejects service with zero duration', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.SERVICES, {
        name: 'Zero Duration',
        duration_minutes: 0,
        base_price_cents: 2500,
      });

      expect(response.ok).toBeFalsy();
    });

    test('rejects service with negative price', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.SERVICES, {
        name: 'Negative Price',
        duration_minutes: 30,
        base_price_cents: -100,
      });

      expect(response.ok).toBeFalsy();
    });
  });
});
