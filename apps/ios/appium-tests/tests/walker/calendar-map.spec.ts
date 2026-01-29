import { CalendarPage } from '../../pages/walker/CalendarPage.js';
import { MapPage } from '../../pages/walker/MapPage.js';
import { expectVisible, loginAsWalker, resetApp, tap } from '../../utils/helpers.js';
import { ids } from '../../utils/ids.js';

describe('Walker - Calendar & Map', () => {
	beforeEach(async () => {
		await resetApp();
		await loginAsWalker();
	});

	it('WCL-001 Calendar Month View', async () => {
		await CalendarPage.open();
		await CalendarPage.expectVisible();
	});

	it('WCL-002 Calendar Week View', async () => {
		await CalendarPage.open();
		await CalendarPage.toggleWeekView();
		await expectVisible(ids.walker.calendarView);
	});

	it('WCL-003 Select Date', async () => {
		await CalendarPage.open();
		await CalendarPage.selectDate('2026-01-28');
		await expectVisible('calendar-bookings-list');
	});

	it('WMP-001 Map Load', async () => {
		await MapPage.open();
		await MapPage.expectVisible();
	});

	it('WMP-002 Location Permission', async () => {
		await MapPage.open();
		await tap(ids.common.permissionAllow);
		await expectVisible(ids.walker.mapView);
	});

	it('WMP-003 Booking Markers', async () => {
		await MapPage.open();
		await MapPage.expectMarkers();
	});
});
