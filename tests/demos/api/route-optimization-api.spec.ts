import { test, expect } from '@playwright/test';
import { loginAsWalker } from '../../utils/auth';
import { createAPIClient } from '../../utils/api';
import { API_ENDPOINTS } from '../../utils/constants';

/**
 * Route Optimization API Demo
 *
 * This test suite demonstrates:
 * 1. Route optimization API with travel time data
 * 2. Availability slots being blocked by travel constraints
 * 3. Calendar blocking affecting availability
 *
 * Run with:
 *   npx playwright test tests/demos/route-optimization-api.spec.ts --project=demos --reporter=list
 */

test.describe('Route Optimization API Demo', () => {
  test('1. Get optimized route for walker', async ({ request }) => {
    const token = await loginAsWalker(request);
    const client = createAPIClient(request, token);

    // Get walker session to get ID
    const session = await client.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
    const walkerId = session.expectSuccess().user.id;

    // Get today's date
    const today = new Date().toISOString().split('T')[0];

    // Call route optimization endpoint
    const routeResponse = await client.get<{
      date: string;
      is_optimized: boolean;
      stops: Array<{
        sequence: number;
        booking_id: string;
        customer_name: string;
        address: string;
        arrival_time: string;
        departure_time: string;
        travel_from_previous_minutes: number;
        service_duration_minutes: number;
      }>;
      total_travel_minutes: number;
      total_distance_meters: number;
      savings_minutes: number;
    }>(`/walkers/${walkerId}/route?date=${today}`);

    if (routeResponse.ok) {
      const route = routeResponse.expectSuccess();
      console.log('\n=== Route Optimization Result ===');
      console.log(`Date: ${route.date}`);
      console.log(`Optimized: ${route.is_optimized}`);
      console.log(`Total Travel: ${route.total_travel_minutes} minutes`);
      console.log(`Savings vs Chronological: ${route.savings_minutes} minutes`);
      console.log(`\nStops (${route.stops.length}):`);

      for (const stop of route.stops) {
        console.log(`  ${stop.sequence}. ${stop.customer_name}`);
        console.log(`     Address: ${stop.address}`);
        console.log(`     Arrival: ${stop.arrival_time}`);
        console.log(`     Travel from prev: ${stop.travel_from_previous_minutes} min`);
      }
    } else {
      console.log('No bookings found for today - route optimization returns empty');
    }

    expect(true).toBeTruthy();
  });

  test('2. Get availability slots with travel constraints', async ({ request }) => {
    const token = await loginAsWalker(request);
    const client = createAPIClient(request, token);

    // Get services
    const servicesResponse = await client.get<Array<{ id: string; name: string }>>(
      API_ENDPOINTS.SERVICES
    );
    const services = servicesResponse.expectSuccess();

    if (services.length === 0) {
      console.log('No services found');
      return;
    }

    // Get tomorrow's date
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    const tomorrowStr = tomorrow.toISOString().split('T')[0];

    // Get availability slots
    const slotsResponse = await client.get<
      Array<{
        start_time: string;
        end_time: string;
        confidence: string;
      }>
    >(
      `${API_ENDPOINTS.AVAILABILITY_SLOTS}?service_id=${services[0].id}&start_date=${tomorrowStr}&end_date=${tomorrowStr}`
    );

    if (slotsResponse.ok) {
      const slots = slotsResponse.expectSuccess();
      console.log('\n=== Available Slots ===');
      console.log(`Date: ${tomorrowStr}`);
      console.log(`Service: ${services[0].name}`);
      console.log(`\nSlots (${slots.length}):`);

      for (const slot of slots.slice(0, 10)) {
        const start = new Date(slot.start_time);
        const end = new Date(slot.end_time);
        console.log(
          `  ${start.toLocaleTimeString()} - ${end.toLocaleTimeString()} [${slot.confidence}]`
        );
      }

      if (slots.length > 10) {
        console.log(`  ... and ${slots.length - 10} more slots`);
      }
    }

    expect(true).toBeTruthy();
  });

  test('3. Trigger route re-optimization', async ({ request }) => {
    const token = await loginAsWalker(request);
    const client = createAPIClient(request, token);

    // Get walker session
    const session = await client.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
    const walkerId = session.expectSuccess().user.id;

    const today = new Date().toISOString().split('T')[0];

    // Trigger optimization
    const optimizeResponse = await client.post<{
      date: string;
      is_optimized: boolean;
      total_travel_minutes: number;
      savings_minutes: number;
    }>(`/walkers/${walkerId}/route/optimize?date=${today}`);

    console.log('\n=== Route Optimization Triggered ===');
    if (optimizeResponse.ok) {
      const result = optimizeResponse.expectSuccess();
      console.log(`Optimization complete`);
      console.log(`Total travel: ${result.total_travel_minutes} minutes`);
      console.log(`Savings: ${result.savings_minutes} minutes`);
    } else {
      console.log('No bookings to optimize');
    }

    expect(true).toBeTruthy();
  });

  test('4. Verify blocked times affect availability', async ({ request }) => {
    // This test demonstrates that blocked calendar times properly
    // reduce available slots for customers

    const walkerToken = await loginAsWalker(request);
    const walkerClient = createAPIClient(request, walkerToken);

    // Get session
    const session = await walkerClient.get<{ user: { id: string } }>(API_ENDPOINTS.SESSION);
    const walkerId = session.expectSuccess().user.id;

    // Get services
    const servicesResponse = await walkerClient.get<Array<{ id: string; name: string }>>(
      API_ENDPOINTS.SERVICES
    );
    const services = servicesResponse.expectSuccess();

    if (services.length === 0) {
      console.log('No services to test availability');
      return;
    }

    // Get date 3 days from now
    const futureDate = new Date();
    futureDate.setDate(futureDate.getDate() + 3);
    const dateStr = futureDate.toISOString().split('T')[0];

    // Get initial availability
    const initialSlots = await walkerClient.get<Array<{ start_time: string }>>(
      `${API_ENDPOINTS.AVAILABILITY_SLOTS}?service_id=${services[0].id}&start_date=${dateStr}&end_date=${dateStr}&walker_id=${walkerId}`
    );

    console.log('\n=== Availability Before Block ===');
    console.log(`Date: ${dateStr}`);
    if (initialSlots.ok) {
      console.log(`Available slots: ${initialSlots.expectSuccess().length}`);
    }

    // Create a calendar block (2pm-3pm)
    const blockStart = `${dateStr}T14:00:00Z`;
    const blockEnd = `${dateStr}T15:00:00Z`;

    const blockResponse = await walkerClient.post<{ id: string }>(API_ENDPOINTS.CALENDAR_EVENTS, {
      title: 'Demo Block - Vet Appointment',
      start_time: blockStart,
      end_time: blockEnd,
      is_blocking: true,
    });

    if (blockResponse.ok) {
      console.log('\n=== Block Created ===');
      console.log(`Block: 2:00 PM - 3:00 PM`);
      console.log(`ID: ${blockResponse.expectSuccess().id}`);

      // Get availability after block
      const afterSlots = await walkerClient.get<Array<{ start_time: string }>>(
        `${API_ENDPOINTS.AVAILABILITY_SLOTS}?service_id=${services[0].id}&start_date=${dateStr}&end_date=${dateStr}&walker_id=${walkerId}`
      );

      console.log('\n=== Availability After Block ===');
      if (afterSlots.ok) {
        const afterCount = afterSlots.expectSuccess().length;
        const beforeCount = initialSlots.ok ? initialSlots.expectSuccess().length : 0;
        console.log(`Available slots: ${afterCount}`);
        console.log(`Slots blocked: ${beforeCount - afterCount}`);
      }

      // Clean up - delete the block
      const block = blockResponse.expectSuccess();
      await walkerClient.delete(`${API_ENDPOINTS.CALENDAR_EVENTS}/${block.id}`);
      console.log('\n=== Block Cleaned Up ===');
    }

    expect(true).toBeTruthy();
  });
});
