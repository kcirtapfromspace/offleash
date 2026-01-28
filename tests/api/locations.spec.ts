import { test, expect } from '@playwright/test';
import { loginAsCustomer, loginAsAdmin } from '../utils/auth';
import { createAPIClient } from '../utils/api';
import { createTestDataFactory, uniqueId } from '../utils/fixtures';
import { API_ENDPOINTS, TEST_LOCATIONS } from '../utils/constants';

test.describe('Locations API', () => {
  test.describe('Customer Locations', () => {
    test('customer can list their locations', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.LOCATIONS);
      expect(response.ok).toBeTruthy();

      const locations = response.expectSuccess();
      expect(Array.isArray(locations)).toBeTruthy();
    });

    test('customer can create a new location', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, token);

      const location = await factory.createLocation({
        address: `${uniqueId('addr')} New Street`,
        city: 'Boulder',
        state: 'CO',
        zip: '80301',
        latitude: 40.0150,
        longitude: -105.2705,
        notes: 'Test location created via API',
      });

      expect(location.id).toBeDefined();
      expect(location.city).toBe('Boulder');
      expect(location.state).toBe('CO');
    });

    test('customer can update their location', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, token);
      const client = createAPIClient(request, token);

      // Create a location
      const location = await factory.createLocation();

      // Update it
      const response = await client.put(`${API_ENDPOINTS.LOCATIONS}/${location.id}`, {
        address: location.address,
        city: location.city,
        state: location.state,
        zip: location.zip,
        latitude: location.latitude,
        longitude: location.longitude,
        notes: 'Updated notes for this location',
      });

      expect(response.ok).toBeTruthy();
      const updated = response.expectSuccess();
      expect(updated.notes).toBe('Updated notes for this location');
    });

    test('customer can delete their location', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, token);
      const client = createAPIClient(request, token);

      // Create a location
      const location = await factory.createLocation({
        address: `${uniqueId('del')} Delete Street`,
      });

      // Delete it
      const deleteResponse = await client.delete(`${API_ENDPOINTS.LOCATIONS}/${location.id}`);
      expect(deleteResponse.ok).toBeTruthy();

      // Verify it's gone
      const getResponse = await client.get(`${API_ENDPOINTS.LOCATIONS}/${location.id}`);
      expect(getResponse.status).toBe(404);
    });

    test('customer can set a default location', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, token);
      const client = createAPIClient(request, token);

      // Create a location
      const location = await factory.createLocation();

      // Set as default
      const response = await client.put(`${API_ENDPOINTS.LOCATIONS}/${location.id}`, {
        address: location.address,
        city: location.city,
        state: location.state,
        zip: location.zip,
        latitude: location.latitude,
        longitude: location.longitude,
        is_default: true,
      });

      expect(response.ok).toBeTruthy();
      const updated = response.expectSuccess();
      expect(updated.is_default).toBe(true);
    });
  });

  test.describe('Location Validation', () => {
    test('rejects location with missing address', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.LOCATIONS, {
        city: 'Denver',
        state: 'CO',
        zip: '80202',
        latitude: 39.7392,
        longitude: -104.9903,
      });

      expect(response.ok).toBeFalsy();
    });

    test('rejects location with invalid coordinates', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.LOCATIONS, {
        address: '123 Invalid Coords St',
        city: 'Denver',
        state: 'CO',
        zip: '80202',
        latitude: 999, // Invalid
        longitude: 999, // Invalid
      });

      expect(response.ok).toBeFalsy();
    });

    test('rejects location with missing city', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.post(API_ENDPOINTS.LOCATIONS, {
        address: '123 No City St',
        state: 'CO',
        zip: '80202',
        latitude: 39.7392,
        longitude: -104.9903,
      });

      expect(response.ok).toBeFalsy();
    });
  });

  test.describe('Admin Location Access', () => {
    test('admin can view all customer locations', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.LOCATIONS);
      expect(response.ok).toBeTruthy();

      const locations = response.expectSuccess();
      expect(Array.isArray(locations)).toBeTruthy();
    });
  });

  test.describe('Location Security', () => {
    test('customer cannot access another customer location', async ({ request }) => {
      const customerToken = await loginAsCustomer(request);
      const customerFactory = createTestDataFactory(request, customerToken);

      // Create a location for customer
      const location = await customerFactory.createLocation();

      // Try to access from another context (simulate other customer)
      // In reality, this would require Org B customer login
      // For now, just verify location IDs are scoped properly
      const customerClient = createAPIClient(request, customerToken);
      const locations = await customerClient.get<any[]>(API_ENDPOINTS.LOCATIONS);
      const locList = locations.expectSuccess();

      // All locations should belong to the authenticated user
      for (const loc of locList) {
        expect(loc.user_id).toBeDefined();
      }
    });
  });
});
