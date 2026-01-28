import { APIRequestContext } from '@playwright/test';
import { createAPIClient, APIClient } from './api';
import { API_ENDPOINTS, TEST_SERVICES, TEST_LOCATIONS, RECURRENCE_FREQUENCY } from './constants';

/**
 * Generate a unique ID for test data
 */
export function uniqueId(prefix: string = 'test'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Generate a unique email for test data
 */
export function uniqueEmail(prefix: string = 'testuser'): string {
  return `${prefix}-${Date.now()}@test.offleash.world`;
}

/**
 * Get a future date (days from now)
 */
export function futureDate(daysFromNow: number): Date {
  const date = new Date();
  date.setDate(date.getDate() + daysFromNow);
  return date;
}

/**
 * Get ISO date string for a date at specific time
 */
export function dateTimeISO(date: Date, hours: number, minutes: number = 0): string {
  const d = new Date(date);
  d.setHours(hours, minutes, 0, 0);
  return d.toISOString();
}

/**
 * Get next occurrence of a day of week (0=Sunday, 6=Saturday)
 */
export function nextDayOfWeek(dayOfWeek: number): Date {
  const today = new Date();
  const currentDay = today.getDay();
  const daysUntil = (dayOfWeek - currentDay + 7) % 7 || 7; // At least 1 day in future
  return futureDate(daysUntil);
}

/**
 * Service data type
 */
export interface ServiceData {
  id: string;
  name: string;
  description: string | null;
  duration_minutes: number;
  price_cents: number;
  is_active: boolean;
}

/**
 * Location data type
 */
export interface LocationData {
  id: string;
  address: string;
  city: string;
  state: string;
  zip: string;
  latitude: number;
  longitude: number;
  notes: string | null;
}

/**
 * Booking data type
 */
export interface BookingData {
  id: string;
  customer_id: string;
  customer_name: string;
  walker_id: string;
  walker_name: string;
  service_id: string;
  service_name: string;
  location_id: string;
  location_address: string;
  status: string;
  scheduled_start: string;
  scheduled_end: string;
  price_cents: number;
  price_display: string;
  notes: string | null;
}

/**
 * Recurring booking series data type
 */
export interface RecurringSeriesData {
  id: string;
  customer_id: string;
  frequency: string;
  day_of_week: number;
  time_of_day: string;
  end_date: string | null;
  total_occurrences: number | null;
  is_active: boolean;
}

/**
 * Test data factory for creating test resources
 */
export class TestDataFactory {
  private client: APIClient;

  constructor(request: APIRequestContext, token: string) {
    this.client = createAPIClient(request, token);
  }

  /**
   * Create a test service
   */
  async createService(overrides: Partial<typeof TEST_SERVICES.walk30> = {}): Promise<ServiceData> {
    const serviceData = {
      ...TEST_SERVICES.walk30,
      name: `${TEST_SERVICES.walk30.name} ${uniqueId()}`,
      ...overrides,
    };

    const response = await this.client.post<ServiceData>(API_ENDPOINTS.SERVICES, serviceData);
    return response.expectSuccess();
  }

  /**
   * Create a test location
   */
  async createLocation(overrides: Partial<typeof TEST_LOCATIONS.home> = {}): Promise<LocationData> {
    const locationData = {
      ...TEST_LOCATIONS.home,
      address: `${uniqueId('addr')} Test Street`,
      ...overrides,
    };

    const response = await this.client.post<LocationData>(API_ENDPOINTS.LOCATIONS, locationData);
    return response.expectSuccess();
  }

  /**
   * Create a test booking
   */
  async createBooking(options: {
    serviceId: string;
    locationId: string;
    walkerId: string;
    scheduledStart?: string;
    scheduledEnd?: string;
    notes?: string;
  }): Promise<BookingData> {
    const tomorrow = futureDate(1);
    const scheduledStart = options.scheduledStart || dateTimeISO(tomorrow, 10, 0);
    const scheduledEnd = options.scheduledEnd || dateTimeISO(tomorrow, 10, 30);

    const bookingData = {
      service_id: options.serviceId,
      location_id: options.locationId,
      walker_id: options.walkerId,
      scheduled_start: scheduledStart,
      scheduled_end: scheduledEnd,
      notes: options.notes || 'Test booking',
    };

    const response = await this.client.post<BookingData>(API_ENDPOINTS.BOOKINGS, bookingData);
    return response.expectSuccess();
  }

  /**
   * Create a recurring booking series
   */
  async createRecurringBooking(options: {
    serviceId: string;
    locationId: string;
    walkerId: string;
    frequency?: string;
    dayOfWeek?: number;
    timeOfDay?: string;
    occurrences?: number;
  }): Promise<{ series: RecurringSeriesData; bookings: BookingData[] }> {
    const startDate = nextDayOfWeek(options.dayOfWeek ?? 1); // Default to next Monday

    const seriesData = {
      service_id: options.serviceId,
      location_id: options.locationId,
      walker_id: options.walkerId,
      frequency: options.frequency || RECURRENCE_FREQUENCY.WEEKLY,
      start_date: startDate.toISOString().split('T')[0],
      time_of_day: options.timeOfDay || '10:00',
      end_condition: {
        type: 'occurrences',
        value: options.occurrences || 4,
      },
    };

    const response = await this.client.post<{ series: RecurringSeriesData; bookings_created: BookingData[] }>(
      API_ENDPOINTS.RECURRING_BOOKINGS,
      seriesData
    );
    const result = response.expectSuccess();
    return {
      series: result.series,
      bookings: result.bookings_created,
    };
  }

  /**
   * Get list of services
   */
  async getServices(): Promise<ServiceData[]> {
    const response = await this.client.get<ServiceData[]>(API_ENDPOINTS.SERVICES);
    return response.expectSuccess();
  }

  /**
   * Get list of bookings
   */
  async getBookings(): Promise<BookingData[]> {
    const response = await this.client.get<BookingData[]>(API_ENDPOINTS.BOOKINGS);
    return response.expectSuccess();
  }

  /**
   * Get walker's bookings
   */
  async getWalkerBookings(): Promise<BookingData[]> {
    const response = await this.client.get<BookingData[]>(API_ENDPOINTS.BOOKINGS_WALKER);
    return response.expectSuccess();
  }

  /**
   * Update booking status
   */
  async updateBookingStatus(bookingId: string, status: string): Promise<BookingData> {
    const response = await this.client.put<BookingData>(`${API_ENDPOINTS.BOOKINGS}/${bookingId}`, { status });
    return response.expectSuccess();
  }

  /**
   * Cancel a booking
   */
  async cancelBooking(bookingId: string): Promise<void> {
    const response = await this.client.post(`${API_ENDPOINTS.BOOKINGS}/${bookingId}/cancel`, {});
    response.expectSuccess();
  }

  /**
   * Get availability slots
   */
  async getAvailabilitySlots(params: {
    serviceId: string;
    startDate: string;
    endDate: string;
    walkerId?: string;
  }): Promise<unknown[]> {
    const url = new URL(API_ENDPOINTS.AVAILABILITY_SLOTS, 'http://localhost');
    url.searchParams.set('service_id', params.serviceId);
    url.searchParams.set('start_date', params.startDate);
    url.searchParams.set('end_date', params.endDate);
    if (params.walkerId) {
      url.searchParams.set('walker_id', params.walkerId);
    }

    const response = await this.client.get<unknown[]>(`${url.pathname}${url.search}`);
    return response.expectSuccess();
  }

  /**
   * Create a calendar block
   */
  async createCalendarBlock(options: {
    title?: string;
    startTime: string;
    endTime: string;
    isBlocking?: boolean;
  }): Promise<unknown> {
    const blockData = {
      title: options.title || 'Test Block',
      start_time: options.startTime,
      end_time: options.endTime,
      event_type: 'block',
      is_blocking: options.isBlocking ?? true,
    };

    const response = await this.client.post(API_ENDPOINTS.CALENDAR_EVENTS, blockData);
    return response.expectSuccess();
  }

  /**
   * Get calendar events
   */
  async getCalendarEvents(startDate: string, endDate: string): Promise<unknown[]> {
    const url = `${API_ENDPOINTS.CALENDAR_EVENTS}?start=${encodeURIComponent(startDate)}&end=${encodeURIComponent(endDate)}`;
    const response = await this.client.get<{ events: unknown[] }>(url);
    const result = response.expectSuccess();
    return result.events;
  }

  /**
   * Delete a calendar event
   */
  async deleteCalendarEvent(eventId: string): Promise<void> {
    const response = await this.client.delete(`${API_ENDPOINTS.CALENDAR_EVENTS}/${eventId}`);
    response.expectSuccess();
  }
}

/**
 * Create a test data factory with authenticated client
 */
export function createTestDataFactory(request: APIRequestContext, token: string): TestDataFactory {
  return new TestDataFactory(request, token);
}
