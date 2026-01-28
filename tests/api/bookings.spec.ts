import { test, expect } from '@playwright/test';
import {
  loginAsCustomer,
  loginAsWalker,
  loginAsAdmin,
} from '../utils/auth';
import {
  createTestDataFactory,
  futureDate,
  dateTimeISO,
  nextDayOfWeek,
} from '../utils/fixtures';
import {
  API_ENDPOINTS,
  BOOKING_STATUS,
  RECURRENCE_FREQUENCY,
} from '../utils/constants';
import { createAPIClient } from '../utils/api';

test.describe('Bookings API', () => {
  test.describe('Customer Bookings', () => {
    test('customer can list their bookings', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.BOOKINGS);
      expect(response.ok).toBeTruthy();

      const bookings = response.expectSuccess();
      expect(Array.isArray(bookings)).toBeTruthy();
    });

    test('customer can create a booking', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, token);

      // Get available services and locations
      const services = await factory.getServices();
      expect(services.length).toBeGreaterThan(0);

      // Get walker info for booking
      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      // Create a location first
      const location = await factory.createLocation();

      // Create booking
      const tomorrow = futureDate(1);
      const booking = await factory.createBooking({
        serviceId: services[0].id,
        locationId: location.id,
        walkerId: walkerId,
        scheduledStart: dateTimeISO(tomorrow, 11, 0),
        scheduledEnd: dateTimeISO(tomorrow, 11, 30),
        notes: 'Test booking created via API',
      });

      expect(booking.id).toBeDefined();
      expect(booking.status).toBe(BOOKING_STATUS.PENDING);
      expect(booking.service_id).toBe(services[0].id);
      expect(booking.location_id).toBe(location.id);
      expect(booking.notes).toBe('Test booking created via API');
    });

    test('customer can view a specific booking', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      // Get existing bookings
      const bookingsResponse = await client.get<any[]>(API_ENDPOINTS.BOOKINGS);
      const bookings = bookingsResponse.expectSuccess();

      if (bookings.length > 0) {
        const bookingId = bookings[0].id;
        const response = await client.get(`${API_ENDPOINTS.BOOKINGS}/${bookingId}`);
        expect(response.ok).toBeTruthy();

        const booking = response.expectSuccess();
        expect(booking.id).toBe(bookingId);
      }
    });

    test('customer can cancel their booking', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, token);
      const client = createAPIClient(request, token);

      // Get services and walker
      const services = await factory.getServices();
      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const location = await factory.createLocation();

      // Create a booking
      const tomorrow = futureDate(2);
      const booking = await factory.createBooking({
        serviceId: services[0].id,
        locationId: location.id,
        walkerId: walkerId,
        scheduledStart: dateTimeISO(tomorrow, 15, 0),
        scheduledEnd: dateTimeISO(tomorrow, 15, 30),
      });

      // Cancel it
      const cancelResponse = await client.post(`${API_ENDPOINTS.BOOKINGS}/${booking.id}/cancel`, {});
      expect(cancelResponse.ok).toBeTruthy();

      // Verify it's cancelled
      const getResponse = await client.get(`${API_ENDPOINTS.BOOKINGS}/${booking.id}`);
      const cancelledBooking = getResponse.expectSuccess();
      expect(cancelledBooking.status).toBe(BOOKING_STATUS.CANCELLED);
    });
  });

  test.describe('Walker Bookings', () => {
    test('walker can list their assigned bookings', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.BOOKINGS_WALKER);
      expect(response.ok).toBeTruthy();

      const bookings = response.expectSuccess();
      expect(Array.isArray(bookings)).toBeTruthy();
    });

    test('walker can update booking status to in_progress', async ({ request }) => {
      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const customerToken = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, customerToken);
      const services = await factory.getServices();
      const location = await factory.createLocation();

      // Create a booking
      const tomorrow = futureDate(1);
      const booking = await factory.createBooking({
        serviceId: services[0].id,
        locationId: location.id,
        walkerId: walkerId,
        scheduledStart: dateTimeISO(tomorrow, 9, 0),
        scheduledEnd: dateTimeISO(tomorrow, 9, 30),
      });

      // Walker updates status
      const updateResponse = await walkerClient.put(`${API_ENDPOINTS.BOOKINGS}/${booking.id}`, {
        status: BOOKING_STATUS.IN_PROGRESS,
      });
      expect(updateResponse.ok).toBeTruthy();

      const updatedBooking = updateResponse.expectSuccess();
      expect(updatedBooking.status).toBe(BOOKING_STATUS.IN_PROGRESS);
    });

    test('walker can complete a booking', async ({ request }) => {
      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const customerToken = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, customerToken);
      const services = await factory.getServices();
      const location = await factory.createLocation();

      // Create and start a booking
      const tomorrow = futureDate(1);
      const booking = await factory.createBooking({
        serviceId: services[0].id,
        locationId: location.id,
        walkerId: walkerId,
        scheduledStart: dateTimeISO(tomorrow, 16, 0),
        scheduledEnd: dateTimeISO(tomorrow, 16, 30),
      });

      // Start booking
      await walkerClient.put(`${API_ENDPOINTS.BOOKINGS}/${booking.id}`, {
        status: BOOKING_STATUS.IN_PROGRESS,
      });

      // Complete booking
      const completeResponse = await walkerClient.put(`${API_ENDPOINTS.BOOKINGS}/${booking.id}`, {
        status: BOOKING_STATUS.COMPLETED,
      });
      expect(completeResponse.ok).toBeTruthy();

      const completedBooking = completeResponse.expectSuccess();
      expect(completedBooking.status).toBe(BOOKING_STATUS.COMPLETED);
    });
  });

  test.describe('Admin Bookings', () => {
    test('admin can view all bookings', async ({ request }) => {
      const token = await loginAsAdmin(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.BOOKINGS);
      expect(response.ok).toBeTruthy();

      const bookings = response.expectSuccess();
      expect(Array.isArray(bookings)).toBeTruthy();
    });

    test('admin can update any booking status', async ({ request }) => {
      const adminToken = await loginAsAdmin(request);
      const adminClient = createAPIClient(request, adminToken);

      // Get a booking
      const bookingsResponse = await adminClient.get<any[]>(API_ENDPOINTS.BOOKINGS);
      const bookings = bookingsResponse.expectSuccess();

      const pendingBooking = bookings.find((b: any) => b.status === BOOKING_STATUS.PENDING);
      if (pendingBooking) {
        const updateResponse = await adminClient.put(
          `${API_ENDPOINTS.BOOKINGS}/${pendingBooking.id}`,
          { status: BOOKING_STATUS.CONFIRMED }
        );
        expect(updateResponse.ok).toBeTruthy();
      }
    });
  });

  test.describe('Recurring Bookings', () => {
    test('customer can create a recurring booking series', async ({ request }) => {
      const customerToken = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, customerToken);

      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const services = await factory.getServices();
      const location = await factory.createLocation();

      const result = await factory.createRecurringBooking({
        serviceId: services[0].id,
        locationId: location.id,
        walkerId: walkerId,
        frequency: RECURRENCE_FREQUENCY.WEEKLY,
        dayOfWeek: 1, // Monday
        timeOfDay: '10:00',
        occurrences: 4,
      });

      expect(result.series).toBeDefined();
      expect(result.series.id).toBeDefined();
      expect(result.series.frequency).toBe(RECURRENCE_FREQUENCY.WEEKLY);
      expect(result.bookings.length).toBeGreaterThan(0);
    });

    test('customer can list their recurring series', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.RECURRING_BOOKINGS);
      expect(response.ok).toBeTruthy();

      const series = response.expectSuccess();
      expect(Array.isArray(series)).toBeTruthy();
    });

    test('customer can preview recurring booking dates', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const startDate = nextDayOfWeek(1); // Next Monday

      const response = await client.post(API_ENDPOINTS.RECURRING_PREVIEW, {
        frequency: RECURRENCE_FREQUENCY.WEEKLY,
        start_date: startDate.toISOString().split('T')[0],
        time_of_day: '10:00',
        end_condition: { type: 'occurrences', value: 4 },
      });

      expect(response.ok).toBeTruthy();
      const preview = response.expectSuccess();
      expect(preview.dates).toBeDefined();
      expect(preview.dates.length).toBe(4);
    });
  });

  test.describe('Booking Validation', () => {
    test('cannot create booking in the past', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);
      const factory = createTestDataFactory(request, token);

      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const services = await factory.getServices();
      const location = await factory.createLocation();

      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);

      const response = await client.post(API_ENDPOINTS.BOOKINGS, {
        service_id: services[0].id,
        location_id: location.id,
        walker_id: walkerId,
        scheduled_start: dateTimeISO(yesterday, 10, 0),
        scheduled_end: dateTimeISO(yesterday, 10, 30),
      });

      expect(response.ok).toBeFalsy();
    });

    test('cannot create booking with invalid service', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);
      const factory = createTestDataFactory(request, token);

      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const location = await factory.createLocation();
      const tomorrow = futureDate(1);

      const response = await client.post(API_ENDPOINTS.BOOKINGS, {
        service_id: '00000000-0000-0000-0000-000000000000',
        location_id: location.id,
        walker_id: walkerId,
        scheduled_start: dateTimeISO(tomorrow, 10, 0),
        scheduled_end: dateTimeISO(tomorrow, 10, 30),
      });

      expect(response.ok).toBeFalsy();
    });
  });
});
