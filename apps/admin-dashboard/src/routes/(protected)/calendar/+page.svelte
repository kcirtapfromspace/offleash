<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';

	export let data: PageData;
	export let form: ActionData;

	// View state
	let view: 'week' | 'day' | 'month' | 'agenda' = 'week';
	let showCreateModal = false;
	let showRecurringModal = false;
	let selectedEvent: typeof data.events[0] | null = null;

	// Create block form state
	let blockTitle = '';
	let blockStartTime = '';
	let blockEndTime = '';
	let blockIsBlocking = true;

	// Recurring block form state
	let recurringTitle = '';
	let recurringStartTime = '12:00';
	let recurringEndTime = '13:00';
	let recurringIsBlocking = true;
	let recurringDays: number[] = [];
	let recurringWeeksAhead = 52; // Default to 1 year
	let recurringIndefinite = true; // Default to indefinite

	// Constants
	const HOUR_START = 6; // 6 AM
	const HOUR_END = 22; // 10 PM
	const HOUR_HEIGHT = 60; // pixels per hour

	// Computed
	$: weekStart = new Date(data.weekStart);
	$: weekEnd = new Date(data.weekEnd);
	$: currentDate = new Date(data.currentDate);
	$: workingHours = data.workingHours || [];
	$: walkers = data.walkers || [];
	$: selectedWalkerId = data.selectedWalkerId;
	$: isAdmin = data.isAdmin;
	$: selectedWalker = walkers.find(w => w.id === selectedWalkerId);

	function selectWalker(walkerId: string) {
		const params = new URLSearchParams(window.location.search);
		params.set('walker', walkerId);
		goto(`/calendar?${params.toString()}`);
	}

	// Working hours helper - check if an hour is within working hours for a given day
	function isWorkingHour(date: Date, hour: number): boolean {
		const dayOfWeek = date.getDay(); // 0 = Sunday, 6 = Saturday
		const daySchedule = workingHours.find(wh => wh.day_of_week === dayOfWeek && wh.is_active);

		if (!daySchedule) {
			// No working hours defined for this day - assume it's a day off
			return false;
		}

		// Parse start and end times (format: "HH:MM")
		const [startHour] = daySchedule.start_time.split(':').map(Number);
		const [endHour] = daySchedule.end_time.split(':').map(Number);

		return hour >= startHour && hour < endHour;
	}

	// Get working hours for a specific day (for display)
	function getWorkingHoursForDay(date: Date): { start: string; end: string } | null {
		const dayOfWeek = date.getDay();
		const daySchedule = workingHours.find(wh => wh.day_of_week === dayOfWeek && wh.is_active);

		if (!daySchedule) return null;

		return {
			start: daySchedule.start_time,
			end: daySchedule.end_time
		};
	}

	$: weekDays = getWeekDays(weekStart);
	$: hours = Array.from({ length: HOUR_END - HOUR_START }, (_, i) => HOUR_START + i);

	function getWeekDays(start: Date): Date[] {
		return Array.from({ length: 7 }, (_, i) => {
			const date = new Date(start);
			date.setDate(date.getDate() + i);
			return date;
		});
	}

	function formatDayHeader(date: Date): string {
		return date.toLocaleDateString('en-US', { weekday: 'short' });
	}

	function formatDayNumber(date: Date): string {
		return date.getDate().toString();
	}

	function formatHour(hour: number): string {
		if (hour === 0) return '12 AM';
		if (hour === 12) return '12 PM';
		if (hour < 12) return `${hour} AM`;
		return `${hour - 12} PM`;
	}

	function formatMonthYear(date: Date): string {
		return date.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
	}

	function isToday(date: Date): boolean {
		const today = new Date();
		return (
			date.getDate() === today.getDate() &&
			date.getMonth() === today.getMonth() &&
			date.getFullYear() === today.getFullYear()
		);
	}

	function navigate(direction: number) {
		const newDate = new Date(currentDate);
		if (view === 'day') {
			newDate.setDate(newDate.getDate() + direction);
		} else if (view === 'week') {
			newDate.setDate(newDate.getDate() + direction * 7);
		} else if (view === 'month') {
			newDate.setMonth(newDate.getMonth() + direction);
		} else {
			// Agenda: navigate by 2 weeks
			newDate.setDate(newDate.getDate() + direction * 14);
		}
		goto(`/calendar?date=${newDate.toISOString().split('T')[0]}`);
	}

	function goToToday() {
		goto('/calendar');
	}

	function goToDay(date: Date) {
		view = 'day';
		goto(`/calendar?date=${date.toISOString().split('T')[0]}`);
	}

	interface MonthDay {
		date: Date;
		isCurrentMonth: boolean;
	}

	function getMonthDays(centerDate: Date): MonthDay[] {
		const year = centerDate.getFullYear();
		const month = centerDate.getMonth();

		// First day of the month
		const firstDay = new Date(year, month, 1);
		// Last day of the month
		const lastDay = new Date(year, month + 1, 0);

		// Start from the Sunday before or on the first day
		const startDate = new Date(firstDay);
		startDate.setDate(startDate.getDate() - startDate.getDay());

		// End on the Saturday after or on the last day
		const endDate = new Date(lastDay);
		endDate.setDate(endDate.getDate() + (6 - endDate.getDay()));

		const days: MonthDay[] = [];
		const current = new Date(startDate);

		while (current <= endDate) {
			days.push({
				date: new Date(current),
				isCurrentMonth: current.getMonth() === month
			});
			current.setDate(current.getDate() + 1);
		}

		return days;
	}

	interface AgendaDay {
		date: Date;
		events: typeof data.events;
	}

	function getAgendaDays(): AgendaDay[] {
		const days: AgendaDay[] = [];
		const start = new Date(currentDate);
		start.setHours(0, 0, 0, 0);

		// Show 14 days from current date
		for (let i = 0; i < 14; i++) {
			const date = new Date(start);
			date.setDate(date.getDate() + i);
			const events = getEventsForDay(date).sort(
				(a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime()
			);
			days.push({ date, events });
		}

		return days;
	}

	function getEventsForDay(date: Date): typeof data.events {
		return data.events.filter((event) => {
			const eventStart = new Date(event.start_time);
			return (
				eventStart.getDate() === date.getDate() &&
				eventStart.getMonth() === date.getMonth() &&
				eventStart.getFullYear() === date.getFullYear()
			);
		});
	}

	function getEventStyle(event: typeof data.events[0]): string {
		const start = new Date(event.start_time);
		const end = new Date(event.end_time);

		const startHour = start.getHours() + start.getMinutes() / 60;
		const endHour = end.getHours() + end.getMinutes() / 60;

		const top = (startHour - HOUR_START) * HOUR_HEIGHT;
		const height = (endHour - startHour) * HOUR_HEIGHT;

		return `top: ${top}px; height: ${Math.max(height, 20)}px;`;
	}

	function getEventColor(event: typeof data.events[0]): string {
		if (event.color) return event.color;

		switch (event.event_type) {
			case 'booking':
				return '#3b82f6'; // blue
			case 'block':
				return '#6b7280'; // gray
			case 'personal':
				return '#8b5cf6'; // purple
			case 'synced':
				return '#10b981'; // green
			default:
				return '#6b7280';
		}
	}

	function getEventTypeLabel(type: string): string {
		switch (type) {
			case 'booking':
				return 'Booking';
			case 'block':
				return 'Blocked';
			case 'personal':
				return 'Personal';
			case 'synced':
				return 'Synced';
			default:
				return type;
		}
	}

	function formatEventTime(event: typeof data.events[0]): string {
		const start = new Date(event.start_time);
		const end = new Date(event.end_time);
		return `${formatTimeShort(start)} - ${formatTimeShort(end)}`;
	}

	function formatTimeShort(date: Date): string {
		return date.toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		});
	}

	function openCreateModal(date: Date, hour: number) {
		const start = new Date(date);
		start.setHours(hour, 0, 0, 0);
		const end = new Date(start);
		end.setHours(hour + 1);

		blockStartTime = start.toISOString().slice(0, 16);
		blockEndTime = end.toISOString().slice(0, 16);
		blockTitle = '';
		blockIsBlocking = true;
		showCreateModal = true;
	}

	function closeCreateModal() {
		showCreateModal = false;
		blockTitle = '';
		blockStartTime = '';
		blockEndTime = '';
	}

	function openRecurringModal() {
		recurringTitle = '';
		recurringStartTime = '12:00';
		recurringEndTime = '13:00';
		recurringIsBlocking = true;
		recurringDays = [];
		recurringWeeksAhead = 52;
		recurringIndefinite = true;
		showRecurringModal = true;
	}

	function closeRecurringModal() {
		showRecurringModal = false;
	}

	function toggleRecurringDay(day: number) {
		if (recurringDays.includes(day)) {
			recurringDays = recurringDays.filter(d => d !== day);
		} else {
			recurringDays = [...recurringDays, day];
		}
	}

	function openEventDetail(event: typeof data.events[0]) {
		selectedEvent = event;
	}

	function closeEventDetail() {
		selectedEvent = null;
	}

	// Current time indicator
	let currentTimeTop = 0;
	function updateCurrentTime() {
		const now = new Date();
		const hours = now.getHours() + now.getMinutes() / 60;
		currentTimeTop = (hours - HOUR_START) * HOUR_HEIGHT;
	}

	updateCurrentTime();
	// Update every minute
	if (typeof window !== 'undefined') {
		setInterval(updateCurrentTime, 60000);
	}
</script>

<div class="flex flex-col h-full">
	<!-- Header -->
	<div class="flex-none p-4 border-b border-gray-200 bg-white">
		<div class="flex items-center justify-between mb-4">
			<div>
				<h1 class="text-2xl font-bold text-gray-900">Calendar</h1>
				<p class="text-gray-600">
					{#if selectedWalker}
						Viewing {selectedWalker.full_name}'s schedule
					{:else}
						Manage schedule and availability
					{/if}
				</p>
			</div>
			<div class="flex items-center gap-3">
				<!-- Walker Selector (Admin only) -->
				{#if isAdmin && walkers.length > 0}
					<select
						value={selectedWalkerId || ''}
						on:change={(e) => selectWalker(e.currentTarget.value)}
						class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
					>
						<option value="" disabled>Select Walker</option>
						{#each walkers as walker}
							<option value={walker.id}>{walker.full_name}</option>
						{/each}
					</select>
				{/if}
				<button
					type="button"
					on:click={openRecurringModal}
					class="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 flex items-center gap-2"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
					</svg>
					Recurring Block
				</button>
				<button
					type="button"
					on:click={() => (showCreateModal = true)}
					class="px-4 py-2 bg-gray-900 text-white rounded-lg hover:bg-gray-800 flex items-center gap-2"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
					</svg>
					Block Time
				</button>
			</div>
		</div>

		{#if form?.error}
			<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
				{form.error}
			</div>
		{/if}

		{#if form?.success}
			<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-4">
				Action completed successfully
			</div>
		{/if}

		{#if data.error}
			<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
				{data.error}
			</div>
		{/if}

		<!-- Toolbar -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-4">
				<!-- Navigation -->
				<div class="flex items-center gap-1">
					<button
						type="button"
						on:click={() => navigate(-1)}
						class="p-2 hover:bg-gray-100 rounded-lg"
						aria-label="Previous"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
						</svg>
					</button>
					<button
						type="button"
						on:click={goToToday}
						class="px-3 py-1.5 text-sm font-medium hover:bg-gray-100 rounded-lg"
					>
						Today
					</button>
					<button
						type="button"
						on:click={() => navigate(1)}
						class="p-2 hover:bg-gray-100 rounded-lg"
						aria-label="Next"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
						</svg>
					</button>
				</div>

				<!-- Current month/year display -->
				<h2 class="text-lg font-semibold text-gray-900">
					{formatMonthYear(currentDate)}
				</h2>
			</div>

			<!-- View switcher -->
			<div class="flex items-center gap-1 bg-gray-100 rounded-lg p-1">
				<button
					type="button"
					on:click={() => (view = 'day')}
					class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors {view === 'day'
						? 'bg-white shadow text-gray-900'
						: 'text-gray-600 hover:text-gray-900'}"
				>
					Day
				</button>
				<button
					type="button"
					on:click={() => (view = 'week')}
					class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors {view === 'week'
						? 'bg-white shadow text-gray-900'
						: 'text-gray-600 hover:text-gray-900'}"
				>
					Week
				</button>
				<button
					type="button"
					on:click={() => (view = 'month')}
					class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors {view === 'month'
						? 'bg-white shadow text-gray-900'
						: 'text-gray-600 hover:text-gray-900'}"
				>
					Month
				</button>
				<button
					type="button"
					on:click={() => (view = 'agenda')}
					class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors {view === 'agenda'
						? 'bg-white shadow text-gray-900'
						: 'text-gray-600 hover:text-gray-900'}"
				>
					Agenda
				</button>
			</div>
		</div>
	</div>

	<!-- Calendar Grid (Week View) -->
	{#if view === 'week'}
		<div class="flex-1 overflow-auto bg-white">
			<!-- Day headers -->
			<div class="sticky top-0 z-10 bg-white border-b border-gray-200">
				<div class="grid grid-cols-8">
					<div class="w-16 border-r border-gray-200"></div>
					{#each weekDays as day}
						{@const dayHours = getWorkingHoursForDay(day)}
						<div class="flex-1 text-center py-2 border-r border-gray-200 last:border-r-0">
							<div class="text-xs text-gray-500 uppercase">{formatDayHeader(day)}</div>
							<div
								class="text-lg font-semibold {isToday(day)
									? 'w-8 h-8 mx-auto rounded-full bg-blue-600 text-white flex items-center justify-center'
									: 'text-gray-900'}"
							>
								{formatDayNumber(day)}
							</div>
							{#if dayHours}
								<div class="text-xs text-gray-400">{dayHours.start}-{dayHours.end}</div>
							{:else}
								<div class="text-xs text-gray-300">Off</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>

			<!-- Time grid -->
			<div class="grid grid-cols-8">
				<!-- Time labels -->
				<div class="w-16 border-r border-gray-200">
					{#each hours as hour}
						<div class="h-[60px] border-b border-gray-100 pr-2 text-right">
							<span class="text-xs text-gray-400 relative -top-2">{formatHour(hour)}</span>
						</div>
					{/each}
				</div>

				<!-- Day columns -->
				{#each weekDays as day, dayIndex}
					<div class="flex-1 border-r border-gray-200 last:border-r-0 relative">
						{#each hours as hour}
							{@const isWorking = isWorkingHour(day, hour)}
							<button
								type="button"
								on:click={() => openCreateModal(day, hour)}
								class="w-full h-[60px] border-b border-gray-100 cursor-pointer {isWorking ? 'hover:bg-gray-50' : 'bg-gray-100 hover:bg-gray-200'}"
								aria-label="Create event at {formatHour(hour)}{isWorking ? '' : ' (outside working hours)'}"
							></button>
						{/each}

						<!-- Events -->
						{#each getEventsForDay(day) as event}
							<button
								type="button"
								on:click|stopPropagation={() => openEventDetail(event)}
								class="absolute left-1 right-1 rounded-lg px-2 py-1 text-xs text-white overflow-hidden cursor-pointer hover:opacity-90 transition-opacity"
								style="{getEventStyle(event)} background-color: {getEventColor(event)};"
							>
								<div class="font-medium truncate">{event.title || getEventTypeLabel(event.event_type)}</div>
								<div class="truncate opacity-80">{formatEventTime(event)}</div>
							</button>
						{/each}

						<!-- Current time indicator -->
						{#if isToday(day) && currentTimeTop >= 0 && currentTimeTop < (HOUR_END - HOUR_START) * HOUR_HEIGHT}
							<div
								class="absolute left-0 right-0 h-0.5 bg-red-500 z-20 pointer-events-none"
								style="top: {currentTimeTop}px;"
							>
								<div class="absolute -left-1 -top-1.5 w-3 h-3 rounded-full bg-red-500"></div>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Day View -->
	{#if view === 'day'}
		{@const dayViewHours = getWorkingHoursForDay(currentDate)}
		<div class="flex-1 overflow-auto bg-white">
			<!-- Day header -->
			<div class="sticky top-0 z-10 bg-white border-b border-gray-200">
				<div class="grid grid-cols-2">
					<div class="w-16 border-r border-gray-200"></div>
					<div class="flex-1 text-center py-3 px-4">
						<div class="text-sm text-gray-500 uppercase">{formatDayHeader(currentDate)}</div>
						<div
							class="text-2xl font-semibold {isToday(currentDate)
								? 'w-10 h-10 mx-auto rounded-full bg-blue-600 text-white flex items-center justify-center'
								: 'text-gray-900'}"
						>
							{formatDayNumber(currentDate)}
						</div>
						{#if dayViewHours}
							<div class="text-xs text-gray-500 mt-1">
								Working: {dayViewHours.start} - {dayViewHours.end}
							</div>
						{:else}
							<div class="text-xs text-gray-400 mt-1">Day off</div>
						{/if}
					</div>
				</div>
			</div>

			<!-- Time grid for single day -->
			<div class="grid grid-cols-2">
				<!-- Time labels -->
				<div class="w-16 border-r border-gray-200">
					{#each hours as hour}
						<div class="h-[60px] border-b border-gray-100 pr-2 text-right">
							<span class="text-xs text-gray-400 relative -top-2">{formatHour(hour)}</span>
						</div>
					{/each}
				</div>

				<!-- Day column -->
				<div class="flex-1 relative">
					{#each hours as hour}
						{@const isWorking = isWorkingHour(currentDate, hour)}
						<button
							type="button"
							on:click={() => openCreateModal(currentDate, hour)}
							class="w-full h-[60px] border-b border-gray-100 cursor-pointer {isWorking ? 'hover:bg-gray-50' : 'bg-gray-100 hover:bg-gray-200'}"
							aria-label="Create event at {formatHour(hour)}{isWorking ? '' : ' (outside working hours)'}"
						></button>
					{/each}

					<!-- Events -->
					{#each getEventsForDay(currentDate) as event}
						<button
							type="button"
							on:click|stopPropagation={() => openEventDetail(event)}
							class="absolute left-2 right-2 rounded-lg px-3 py-2 text-sm text-white overflow-hidden cursor-pointer hover:opacity-90 transition-opacity"
							style="{getEventStyle(event)} background-color: {getEventColor(event)};"
						>
							<div class="font-medium">{event.title || getEventTypeLabel(event.event_type)}</div>
							<div class="opacity-80">{formatEventTime(event)}</div>
							{#if event.description}
								<div class="text-xs mt-1 opacity-70 truncate">{event.description}</div>
							{/if}
						</button>
					{/each}

					<!-- Current time indicator -->
					{#if isToday(currentDate) && currentTimeTop >= 0 && currentTimeTop < (HOUR_END - HOUR_START) * HOUR_HEIGHT}
						<div
							class="absolute left-0 right-0 h-0.5 bg-red-500 z-20 pointer-events-none"
							style="top: {currentTimeTop}px;"
						>
							<div class="absolute -left-1 -top-1.5 w-3 h-3 rounded-full bg-red-500"></div>
						</div>
					{/if}
				</div>
			</div>
		</div>
	{/if}

	<!-- Month View -->
	{#if view === 'month'}
		<div class="flex-1 overflow-auto bg-white p-4">
			<!-- Month header -->
			<div class="grid grid-cols-7 mb-2">
				{#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as day}
					<div class="text-center text-sm font-medium text-gray-500 py-2">{day}</div>
				{/each}
			</div>

			<!-- Month grid -->
			<div class="grid grid-cols-7 gap-1">
				{#each getMonthDays(currentDate) as day}
					{@const dayHours = getWorkingHoursForDay(day.date)}
					{@const isDayOff = !dayHours}
					<button
						type="button"
						on:click={() => goToDay(day.date)}
						class="min-h-[100px] p-2 border rounded-lg text-left transition-colors {day.isCurrentMonth
							? isDayOff ? 'bg-gray-100 hover:bg-gray-150' : 'bg-white hover:bg-gray-50'
							: 'bg-gray-50 text-gray-400'} {isToday(day.date) ? 'ring-2 ring-blue-500' : ''}"
					>
						<div class="flex items-center justify-between">
							<span class="font-medium {isToday(day.date) ? 'text-blue-600' : ''}">{day.date.getDate()}</span>
							{#if isDayOff && day.isCurrentMonth}
								<span class="text-xs text-gray-400">Off</span>
							{/if}
						</div>
						<div class="mt-1 space-y-1">
							{#each getEventsForDay(day.date).slice(0, 3) as event}
								<div
									class="text-xs px-1 py-0.5 rounded truncate text-white"
									style="background-color: {getEventColor(event)};"
								>
									{event.title || getEventTypeLabel(event.event_type)}
								</div>
							{/each}
							{#if getEventsForDay(day.date).length > 3}
								<div class="text-xs text-gray-500">+{getEventsForDay(day.date).length - 3} more</div>
							{/if}
						</div>
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Agenda View -->
	{#if view === 'agenda'}
		<div class="flex-1 overflow-auto bg-white">
			<div class="max-w-3xl mx-auto p-4">
				{#each getAgendaDays() as agendaDay}
					{@const dayHours = getWorkingHoursForDay(agendaDay.date)}
					<div class="mb-6">
						<div class="sticky top-0 bg-white py-2 border-b border-gray-200 z-10">
							<div class="flex items-center justify-between">
								<h3 class="font-semibold text-gray-900">
									{agendaDay.date.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })}
									{#if isToday(agendaDay.date)}
										<span class="ml-2 text-sm text-blue-600 font-normal">Today</span>
									{/if}
								</h3>
								{#if dayHours}
									<span class="text-sm text-gray-500">Working: {dayHours.start} - {dayHours.end}</span>
								{:else}
									<span class="text-sm text-gray-400">Day off</span>
								{/if}
							</div>
						</div>
						{#if agendaDay.events.length === 0}
							<div class="py-4 text-gray-500 text-sm">No events scheduled</div>
						{:else}
							<div class="divide-y divide-gray-100">
								{#each agendaDay.events as event}
									<button
										type="button"
										on:click={() => openEventDetail(event)}
										class="w-full py-3 flex items-start gap-4 hover:bg-gray-50 rounded-lg px-2 -mx-2 text-left"
									>
										<div
											class="w-1 h-12 rounded-full flex-shrink-0"
											style="background-color: {getEventColor(event)};"
										></div>
										<div class="flex-1 min-w-0">
											<div class="font-medium text-gray-900">{event.title || getEventTypeLabel(event.event_type)}</div>
											<div class="text-sm text-gray-500">{formatEventTime(event)}</div>
											{#if event.description}
												<div class="text-sm text-gray-500 truncate mt-1">{event.description}</div>
											{/if}
										</div>
										<span
											class="px-2 py-1 text-xs font-medium rounded-full flex-shrink-0"
											class:bg-blue-100={event.event_type === 'booking'}
											class:text-blue-800={event.event_type === 'booking'}
											class:bg-gray-100={event.event_type === 'block'}
											class:text-gray-800={event.event_type === 'block'}
											class:bg-purple-100={event.event_type === 'personal'}
											class:text-purple-800={event.event_type === 'personal'}
											class:bg-green-100={event.event_type === 'synced'}
											class:text-green-800={event.event_type === 'synced'}
										>
											{getEventTypeLabel(event.event_type)}
										</span>
									</button>
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>

<!-- Create Block Modal -->
{#if showCreateModal}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={closeCreateModal} on:keydown={(e) => e.key === 'Escape' && closeCreateModal()} role="dialog" aria-modal="true">
		<div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4" on:click|stopPropagation on:keydown|stopPropagation>
			<div class="p-4 border-b border-gray-200">
				<h3 class="text-lg font-semibold text-gray-900">Block Time</h3>
			</div>
			<form method="POST" action="?/createBlock" use:enhance={() => {
				return async ({ result }) => {
					if (result.type === 'success') {
						closeCreateModal();
						// Reload page to show new event
						window.location.reload();
					}
				};
			}}>
				<div class="p-4 space-y-4">
					<div>
						<label for="title" class="block text-sm font-medium text-gray-700 mb-1">
							Title (optional)
						</label>
						<input
							type="text"
							id="title"
							name="title"
							bind:value={blockTitle}
							placeholder="e.g., Lunch break"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
						/>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="start_time" class="block text-sm font-medium text-gray-700 mb-1">
								Start Time
							</label>
							<input
								type="datetime-local"
								id="start_time"
								name="start_time"
								bind:value={blockStartTime}
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
							/>
						</div>
						<div>
							<label for="end_time" class="block text-sm font-medium text-gray-700 mb-1">
								End Time
							</label>
							<input
								type="datetime-local"
								id="end_time"
								name="end_time"
								bind:value={blockEndTime}
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
							/>
						</div>
					</div>

					<div class="flex items-center gap-2">
						<input
							type="checkbox"
							id="is_blocking"
							name="is_blocking"
							value="true"
							checked={blockIsBlocking}
							on:change={() => (blockIsBlocking = !blockIsBlocking)}
							class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
						/>
						<label for="is_blocking" class="text-sm text-gray-700">
							Block booking availability during this time
						</label>
					</div>
				</div>

				<div class="p-4 border-t border-gray-200 flex justify-end gap-3">
					<button
						type="button"
						on:click={closeCreateModal}
						class="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg"
					>
						Cancel
					</button>
					<button
						type="submit"
						class="px-4 py-2 bg-gray-900 text-white rounded-lg hover:bg-gray-800"
					>
						Create Block
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<!-- Event Detail Modal -->
{#if selectedEvent}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={closeEventDetail} on:keydown={(e) => e.key === 'Escape' && closeEventDetail()} role="dialog" aria-modal="true">
		<div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4" on:click|stopPropagation on:keydown|stopPropagation>
			<div class="p-4 border-b border-gray-200 flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div
						class="w-4 h-4 rounded-full"
						style="background-color: {getEventColor(selectedEvent)};"
					></div>
					<h3 class="text-lg font-semibold text-gray-900">
						{selectedEvent.title || getEventTypeLabel(selectedEvent.event_type)}
					</h3>
				</div>
				<button
					type="button"
					on:click={closeEventDetail}
					class="text-gray-400 hover:text-gray-600"
					aria-label="Close"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
			<div class="p-4 space-y-4">
				<div class="flex items-start gap-3">
					<svg class="w-5 h-5 text-gray-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
					<div>
						<p class="font-medium text-gray-900">{formatEventTime(selectedEvent)}</p>
						<p class="text-sm text-gray-500">
							{new Date(selectedEvent.start_time).toLocaleDateString('en-US', {
								weekday: 'long',
								month: 'long',
								day: 'numeric',
								year: 'numeric'
							})}
						</p>
					</div>
				</div>

				{#if selectedEvent.description}
					<div class="flex items-start gap-3">
						<svg class="w-5 h-5 text-gray-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
						</svg>
						<p class="text-gray-700">{selectedEvent.description}</p>
					</div>
				{/if}

				<div class="flex items-center gap-2">
					<span
						class="px-2 py-1 text-xs font-medium rounded-full"
						class:bg-blue-100={selectedEvent.event_type === 'booking'}
						class:text-blue-800={selectedEvent.event_type === 'booking'}
						class:bg-gray-100={selectedEvent.event_type === 'block'}
						class:text-gray-800={selectedEvent.event_type === 'block'}
						class:bg-purple-100={selectedEvent.event_type === 'personal'}
						class:text-purple-800={selectedEvent.event_type === 'personal'}
						class:bg-green-100={selectedEvent.event_type === 'synced'}
						class:text-green-800={selectedEvent.event_type === 'synced'}
					>
						{getEventTypeLabel(selectedEvent.event_type)}
					</span>
					{#if selectedEvent.is_blocking}
						<span class="px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded-full">
							Blocks Availability
						</span>
					{/if}
				</div>
			</div>

			{#if selectedEvent.event_type !== 'booking' && selectedEvent.event_type !== 'synced'}
				<div class="p-4 border-t border-gray-200 flex justify-end gap-3">
					<form method="POST" action="?/deleteEvent" use:enhance={() => {
						return async ({ result }) => {
							if (result.type === 'success') {
								closeEventDetail();
								window.location.reload();
							}
						};
					}}>
						<input type="hidden" name="event_id" value={selectedEvent.id} />
						<button
							type="submit"
							class="px-4 py-2 text-red-600 hover:bg-red-50 rounded-lg"
						>
							Delete
						</button>
					</form>
					<button
						type="button"
						on:click={closeEventDetail}
						class="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg"
					>
						Close
					</button>
				</div>
			{:else}
				<div class="p-4 border-t border-gray-200">
					<p class="text-sm text-gray-500">
						{#if selectedEvent.event_type === 'booking'}
							View booking details in the Bookings page.
						{:else}
							This event is synced from an external calendar. Edit it in the source calendar.
						{/if}
					</p>
				</div>
			{/if}
		</div>
	</div>
{/if}

<!-- Recurring Block Modal -->
{#if showRecurringModal}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={closeRecurringModal} on:keydown={(e) => e.key === 'Escape' && closeRecurringModal()} role="dialog" aria-modal="true">
		<div class="bg-white rounded-xl shadow-xl w-full max-w-lg mx-4" on:click|stopPropagation on:keydown|stopPropagation>
			<div class="p-4 border-b border-gray-200">
				<h3 class="text-lg font-semibold text-gray-900">Create Recurring Block</h3>
				<p class="text-sm text-gray-500 mt-1">Set up a recurring break like lunch or gym time</p>
			</div>
			<form method="POST" action="?/createRecurringBlock" use:enhance={() => {
				return async ({ result }) => {
					if (result.type === 'success') {
						closeRecurringModal();
						window.location.reload();
					}
				};
			}}>
				<div class="p-4 space-y-4">
					<div>
						<label for="recurring_title" class="block text-sm font-medium text-gray-700 mb-1">
							Title
						</label>
						<input
							type="text"
							id="recurring_title"
							name="title"
							bind:value={recurringTitle}
							placeholder="e.g., Lunch break, Gym time"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
						/>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="recurring_start_time" class="block text-sm font-medium text-gray-700 mb-1">
								Start Time
							</label>
							<input
								type="time"
								id="recurring_start_time"
								name="start_time"
								bind:value={recurringStartTime}
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
							/>
						</div>
						<div>
							<label for="recurring_end_time" class="block text-sm font-medium text-gray-700 mb-1">
								End Time
							</label>
							<input
								type="time"
								id="recurring_end_time"
								name="end_time"
								bind:value={recurringEndTime}
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
							/>
						</div>
					</div>

					<div>
						<label class="block text-sm font-medium text-gray-700 mb-2">
							Days of the Week
						</label>
						<div class="flex flex-wrap gap-2">
							{#each [
								{ day: 0, label: 'Sun' },
								{ day: 1, label: 'Mon' },
								{ day: 2, label: 'Tue' },
								{ day: 3, label: 'Wed' },
								{ day: 4, label: 'Thu' },
								{ day: 5, label: 'Fri' },
								{ day: 6, label: 'Sat' }
							] as { day, label }}
								<button
									type="button"
									on:click={() => toggleRecurringDay(day)}
									class="px-3 py-2 rounded-lg border transition-colors {recurringDays.includes(day)
										? 'bg-blue-600 text-white border-blue-600'
										: 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'}"
								>
									{label}
								</button>
								{#if recurringDays.includes(day)}
									<input type="hidden" name="days_of_week" value={day} />
								{/if}
							{/each}
						</div>
					</div>

					<div>
						<label class="block text-sm font-medium text-gray-700 mb-2">
							Duration
						</label>
						<div class="space-y-3">
							<label class="flex items-center gap-2">
								<input
									type="radio"
									name="duration_type"
									checked={recurringIndefinite}
									on:change={() => (recurringIndefinite = true)}
									class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500"
								/>
								<span class="text-sm text-gray-700">Indefinite (creates 1 year at a time, auto-extends)</span>
							</label>
							<label class="flex items-center gap-2">
								<input
									type="radio"
									name="duration_type"
									checked={!recurringIndefinite}
									on:change={() => (recurringIndefinite = false)}
									class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500"
								/>
								<span class="text-sm text-gray-700">Fixed duration:</span>
								<select
									id="weeks_ahead"
									name="weeks_ahead"
									bind:value={recurringWeeksAhead}
									disabled={recurringIndefinite}
									class="px-3 py-1.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100 disabled:text-gray-500"
								>
									<option value={4}>4 weeks</option>
									<option value={8}>8 weeks</option>
									<option value={12}>12 weeks</option>
									<option value={26}>26 weeks</option>
									<option value={52}>52 weeks</option>
								</select>
							</label>
						</div>
						{#if recurringIndefinite}
							<input type="hidden" name="weeks_ahead" value="52" />
							<input type="hidden" name="indefinite" value="true" />
						{/if}
					</div>

					<div class="flex items-center gap-2">
						<input
							type="checkbox"
							id="recurring_is_blocking"
							name="is_blocking"
							value="true"
							checked={recurringIsBlocking}
							on:change={() => (recurringIsBlocking = !recurringIsBlocking)}
							class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
						/>
						<label for="recurring_is_blocking" class="text-sm text-gray-700">
							Block booking availability during this time
						</label>
					</div>

					{#if recurringDays.length > 0}
						<div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
							<p class="text-sm text-blue-800">
								{#if recurringIndefinite}
									This will create <strong>{recurringDays.length * 52}</strong> blocked time slots for the next year
									({recurringDays.length} days/week). More will be auto-created as needed.
								{:else}
									This will create <strong>{recurringDays.length * recurringWeeksAhead}</strong> blocked time slots
									({recurringDays.length} days/week × {recurringWeeksAhead} weeks)
								{/if}
							</p>
						</div>
					{/if}
				</div>

				<div class="p-4 border-t border-gray-200 flex justify-end gap-3">
					<button
						type="button"
						on:click={closeRecurringModal}
						class="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={recurringDays.length === 0}
						class="px-4 py-2 bg-gray-900 text-white rounded-lg hover:bg-gray-800 disabled:bg-gray-300 disabled:cursor-not-allowed"
					>
						Create Recurring Block
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<style>
	/* Ensure the calendar takes full height */
	:global(.calendar-container) {
		height: calc(100vh - 64px);
	}
</style>
