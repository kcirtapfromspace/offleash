import { test, expect } from '@playwright/test';
import { loginAsCustomer, loginAsWalker, loginAsAdmin } from '../utils/auth';
import { createAPIClient } from '../utils/api';
import { createTestDataFactory, futureDate, dateTimeISO } from '../utils/fixtures';
import { API_ENDPOINTS } from '../utils/constants';

test.describe('Availability API', () => {
  test.describe('Get Availability Slots', () => {
    test('customer can get availability slots for a service', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, token);

      const services = await factory.getServices();
      expect(services.length).toBeGreaterThan(0);

      const tomorrow = futureDate(1);
      const nextWeek = futureDate(7);

      const slots = await factory.getAvailabilitySlots({
        serviceId: services[0].id,
        startDate: tomorrow.toISOString().split('T')[0],
        endDate: nextWeek.toISOString().split('T')[0],
      });

      expect(Array.isArray(slots)).toBeTruthy();
      // Slots should exist during working hours
      expect(slots.length).toBeGreaterThan(0);
    });

    test('customer can filter availability by specific walker', async ({ request }) => {
      const customerToken = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, customerToken);

      // Get walker ID
      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const services = await factory.getServices();
      const tomorrow = futureDate(1);
      const dayAfter = futureDate(2);

      const slots = await factory.getAvailabilitySlots({
        serviceId: services[0].id,
        startDate: tomorrow.toISOString().split('T')[0],
        endDate: dayAfter.toISOString().split('T')[0],
        walkerId: walkerId,
      });

      expect(Array.isArray(slots)).toBeTruthy();
    });

    test('availability respects walker working hours', async ({ request }) => {
      const customerToken = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, customerToken);

      const services = await factory.getServices();
      const tomorrow = futureDate(1);
      const nextWeek = futureDate(7);

      const slots = await factory.getAvailabilitySlots({
        serviceId: services[0].id,
        startDate: tomorrow.toISOString().split('T')[0],
        endDate: nextWeek.toISOString().split('T')[0],
      });

      // Walker working hours are 8am-6pm (from seed data)
      for (const slot of slots) {
        const slotDate = new Date(slot.start_time);
        const hour = slotDate.getHours();
        expect(hour).toBeGreaterThanOrEqual(8);
        expect(hour).toBeLessThan(18);
      }
    });

    test('availability excludes booked times', async ({ request }) => {
      const customerToken = await loginAsCustomer(request);
      const factory = createTestDataFactory(request, customerToken);
      const client = createAPIClient(request, customerToken);

      // Get walker
      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const services = await factory.getServices();
      const location = await factory.createLocation();

      // Book a specific time
      const threeDaysOut = futureDate(3);
      const bookedStart = dateTimeISO(threeDaysOut, 10, 0);
      const bookedEnd = dateTimeISO(threeDaysOut, 10, 30);

      await factory.createBooking({
        serviceId: services[0].id,
        locationId: location.id,
        walkerId: walkerId,
        scheduledStart: bookedStart,
        scheduledEnd: bookedEnd,
      });

      // Get availability for that day
      const slots = await factory.getAvailabilitySlots({
        serviceId: services[0].id,
        startDate: threeDaysOut.toISOString().split('T')[0],
        endDate: threeDaysOut.toISOString().split('T')[0],
        walkerId: walkerId,
      });

      // The booked slot should not be available
      const bookedSlot = slots.find((s: any) => {
        const slotStart = new Date(s.start_time);
        const bookedStartDate = new Date(bookedStart);
        return slotStart.getTime() === bookedStartDate.getTime();
      });

      expect(bookedSlot).toBeUndefined();
    });

    test('returns empty slots for invalid service', async ({ request }) => {
      const token = await loginAsCustomer(request);
      const client = createAPIClient(request, token);

      const tomorrow = futureDate(1);
      const url = `${API_ENDPOINTS.AVAILABILITY_SLOTS}?service_id=00000000-0000-0000-0000-000000000000&start_date=${tomorrow.toISOString().split('T')[0]}&end_date=${tomorrow.toISOString().split('T')[0]}`;

      const response = await client.get(url);
      // Should either be empty or return an error
      if (response.ok) {
        const slots = response.expectSuccess();
        expect(Array.isArray(slots)).toBeTruthy();
        expect(slots.length).toBe(0);
      }
    });
  });

  test.describe('Calendar Events', () => {
    test('walker can view their calendar events', async ({ request }) => {
      const token = await loginAsWalker(request);
      const factory = createTestDataFactory(request, token);

      const tomorrow = futureDate(1);
      const nextWeek = futureDate(7);

      const events = await factory.getCalendarEvents(
        tomorrow.toISOString(),
        nextWeek.toISOString()
      );

      expect(Array.isArray(events)).toBeTruthy();
    });

    test('walker can create a calendar block', async ({ request }) => {
      const token = await loginAsWalker(request);
      const factory = createTestDataFactory(request, token);

      const threeDaysOut = futureDate(3);
      const block = await factory.createCalendarBlock({
        title: 'Personal appointment',
        startTime: dateTimeISO(threeDaysOut, 12, 0),
        endTime: dateTimeISO(threeDaysOut, 13, 0),
        isBlocking: true,
      });

      expect(block).toBeDefined();
      expect(block.id).toBeDefined();
    });

    test('calendar blocks affect availability', async ({ request }) => {
      const walkerToken = await loginAsWalker(request);
      const walkerFactory = createTestDataFactory(request, walkerToken);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const customerToken = await loginAsCustomer(request);
      const customerFactory = createTestDataFactory(request, customerToken);

      const services = await customerFactory.getServices();

      // Create a block on a specific day
      const fourDaysOut = futureDate(4);
      await walkerFactory.createCalendarBlock({
        title: 'Blocked time',
        startTime: dateTimeISO(fourDaysOut, 14, 0),
        endTime: dateTimeISO(fourDaysOut, 15, 0),
        isBlocking: true,
      });

      // Check availability
      const slots = await customerFactory.getAvailabilitySlots({
        serviceId: services[0].id,
        startDate: fourDaysOut.toISOString().split('T')[0],
        endDate: fourDaysOut.toISOString().split('T')[0],
        walkerId: walkerId,
      });

      // The blocked time should not be available
      const blockedSlot = slots.find((s: any) => {
        const slotStart = new Date(s.start_time);
        return slotStart.getHours() === 14 && slotStart.getMinutes() === 0;
      });

      expect(blockedSlot).toBeUndefined();
    });

    test('walker can delete a calendar event', async ({ request }) => {
      const token = await loginAsWalker(request);
      const factory = createTestDataFactory(request, token);

      // Create a block
      const fiveDaysOut = futureDate(5);
      const block = await factory.createCalendarBlock({
        title: 'Temporary block',
        startTime: dateTimeISO(fiveDaysOut, 11, 0),
        endTime: dateTimeISO(fiveDaysOut, 12, 0),
      });

      // Delete it
      await factory.deleteCalendarEvent(block.id);

      // Verify it's gone
      const events = await factory.getCalendarEvents(
        fiveDaysOut.toISOString(),
        fiveDaysOut.toISOString()
      );

      const deletedBlock = events.find((e: any) => e.id === block.id);
      expect(deletedBlock).toBeUndefined();
    });
  });

  test.describe('Working Hours', () => {
    test('walker can view their working hours', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      const response = await client.get(API_ENDPOINTS.WORKING_HOURS);
      expect(response.ok).toBeTruthy();

      const hours = response.expectSuccess();
      expect(Array.isArray(hours)).toBeTruthy();
    });

    test('walker can update their working hours', async ({ request }) => {
      const token = await loginAsWalker(request);
      const client = createAPIClient(request, token);

      // Update Monday's hours
      const response = await client.put(API_ENDPOINTS.WORKING_HOURS, {
        day_of_week: 1, // Monday
        start_time: '09:00',
        end_time: '17:00',
        is_active: true,
      });

      expect(response.ok).toBeTruthy();
    });

    test('admin can view any walker working hours', async ({ request }) => {
      const walkerToken = await loginAsWalker(request);
      const walkerClient = createAPIClient(request, walkerToken);
      const walkerSession = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
      const walkerId = walkerSession.expectSuccess().user.id;

      const adminToken = await loginAsAdmin(request);
      const adminClient = createAPIClient(request, adminToken);

      const response = await adminClient.get(`${API_ENDPOINTS.WORKING_HOURS}?walker_id=${walkerId}`);
      expect(response.ok).toBeTruthy();
    });
  });
});
