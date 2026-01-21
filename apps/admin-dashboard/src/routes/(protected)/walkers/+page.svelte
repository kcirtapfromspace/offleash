<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';

	export let data: PageData;
	export let form: ActionData;

	let showCreateModal = false;
	let showScheduleModal = false;
	let selectedWalker: (typeof data.walkers)[0] | null = null;
	let isSubmitting = false;

	const dayNames = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

	// Default schedule template
	const defaultSchedule = dayNames.map((_, i) => ({
		day_of_week: i,
		start_time: '09:00',
		end_time: '17:00',
		is_active: i >= 1 && i <= 5 // Mon-Fri active by default
	}));

	let editSchedule = [...defaultSchedule];

	function formatDate(isoString: string): string {
		return new Date(isoString).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatTime(timeString: string): string {
		const [hours, minutes] = timeString.split(':');
		const hour = parseInt(hours);
		const ampm = hour >= 12 ? 'PM' : 'AM';
		const displayHour = hour % 12 || 12;
		return `${displayHour}:${minutes} ${ampm}`;
	}

	function getWorkingDays(workingHours: { day_of_week: number; is_active: boolean }[]): string {
		if (!workingHours || workingHours.length === 0) return 'Not set';
		const activeDays = workingHours.filter((wh) => wh.is_active);
		if (activeDays.length === 0) return 'None';
		const days = [...new Set(activeDays.map((wh) => wh.day_of_week))].sort();
		return days.map((d) => dayNames[d].substring(0, 3)).join(', ');
	}

	function openScheduleModal(walker: (typeof data.walkers)[0]) {
		selectedWalker = walker;
		// Populate with walker's existing hours or defaults
		editSchedule = dayNames.map((_, i) => {
			const existing = walker.working_hours?.find((wh) => wh.day_of_week === i);
			if (existing) {
				return {
					day_of_week: i,
					start_time: existing.start_time,
					end_time: existing.end_time,
					is_active: existing.is_active
				};
			}
			return {
				day_of_week: i,
				start_time: '09:00',
				end_time: '17:00',
				is_active: i >= 1 && i <= 5
			};
		});
		showScheduleModal = true;
	}

	function closeScheduleModal() {
		showScheduleModal = false;
		selectedWalker = null;
	}

	// Close modal on success
	$: if (form?.success) {
		showCreateModal = false;
		showScheduleModal = false;
		isSubmitting = false;
	}
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-900">Walkers</h1>
			<p class="text-gray-600">Manage your dog walkers</p>
		</div>
		<button
			on:click={() => (showCreateModal = true)}
			class="inline-flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium"
		>
			<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
			</svg>
			Add Walker
		</button>
	</div>

	{#if form?.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{form.error}
		</div>
	{/if}

	{#if form?.success}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			{form.message || 'Success!'}
		</div>
	{/if}

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	<!-- Stats -->
	<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">Total Walkers</p>
			<p class="text-2xl font-bold text-gray-900">{data.walkers.length}</p>
		</div>
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">With Schedule</p>
			<p class="text-2xl font-bold text-green-600">
				{data.walkers.filter((w) => w.working_hours && w.working_hours.length > 0).length}
			</p>
		</div>
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">No Schedule</p>
			<p class="text-2xl font-bold text-amber-600">
				{data.walkers.filter((w) => !w.working_hours || w.working_hours.length === 0).length}
			</p>
		</div>
	</div>

	<!-- Walkers Grid -->
	{#if data.walkers.length === 0}
		<div class="bg-white border border-gray-200 rounded-xl p-8 text-center">
			<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
				/>
			</svg>
			<p class="text-gray-500 mb-4">No walkers yet</p>
			<button
				on:click={() => (showCreateModal = true)}
				class="inline-flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium"
			>
				<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
				</svg>
				Add Your First Walker
			</button>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each data.walkers as walker}
				<div class="bg-white border border-gray-200 rounded-xl overflow-hidden hover:shadow-md transition-shadow">
					<div class="p-6">
						<!-- Header -->
						<div class="flex items-start justify-between mb-4">
							<div class="flex items-center">
								<div class="w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center mr-3">
									<span class="text-purple-600 font-semibold text-lg">
										{walker.first_name[0]}{walker.last_name[0]}
									</span>
								</div>
								<div>
									<h3 class="font-semibold text-gray-900">{walker.first_name} {walker.last_name}</h3>
									<p class="text-sm text-gray-500">{walker.email}</p>
								</div>
							</div>
							{#if walker.working_hours && walker.working_hours.length > 0}
								<span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800">
									Schedule Set
								</span>
							{:else}
								<span class="px-2 py-1 text-xs font-medium rounded-full bg-amber-100 text-amber-800">
									No Schedule
								</span>
							{/if}
						</div>

						<!-- Details -->
						<div class="space-y-2 text-sm">
							{#if walker.phone}
								<div class="flex items-center text-gray-600">
									<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z"
										/>
									</svg>
									{walker.phone}
								</div>
							{/if}
							<div class="flex items-center text-gray-600">
								<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
									/>
								</svg>
								Working: {getWorkingDays(walker.working_hours || [])}
							</div>
							<div class="flex items-center text-gray-500">
								<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
									/>
								</svg>
								Joined {formatDate(walker.created_at)}
							</div>
						</div>
					</div>

					<!-- Actions -->
					<div class="px-6 py-3 bg-gray-50 border-t border-gray-200 flex gap-4">
						<button
							on:click={() => openScheduleModal(walker)}
							class="text-sm font-medium text-purple-600 hover:text-purple-800"
						>
							{walker.working_hours && walker.working_hours.length > 0 ? 'Edit Schedule' : 'Set Schedule'}
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Create Walker Modal -->
{#if showCreateModal}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-xl max-w-md w-full p-6">
			<div class="flex items-center justify-between mb-6">
				<h2 class="text-xl font-bold text-gray-900">Add New Walker</h2>
				<button on:click={() => (showCreateModal = false)} class="text-gray-400 hover:text-gray-600">
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<form
				method="POST"
				action="?/createWalker"
				use:enhance={() => {
					isSubmitting = true;
					return async ({ update }) => {
						isSubmitting = false;
						await update();
					};
				}}
			>
				<div class="space-y-4">
					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="first_name" class="block text-sm font-medium text-gray-700 mb-1">First Name</label>
							<input
								type="text"
								id="first_name"
								name="first_name"
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
								placeholder="John"
							/>
						</div>
						<div>
							<label for="last_name" class="block text-sm font-medium text-gray-700 mb-1">Last Name</label>
							<input
								type="text"
								id="last_name"
								name="last_name"
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
								placeholder="Doe"
							/>
						</div>
					</div>

					<div>
						<label for="email" class="block text-sm font-medium text-gray-700 mb-1">Email</label>
						<input
							type="email"
							id="email"
							name="email"
							required
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="john@example.com"
						/>
					</div>

					<div>
						<label for="phone" class="block text-sm font-medium text-gray-700 mb-1">Phone (Optional)</label>
						<input
							type="tel"
							id="phone"
							name="phone"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="(555) 123-4567"
						/>
					</div>

					<div>
						<label for="password" class="block text-sm font-medium text-gray-700 mb-1">Initial Password</label>
						<input
							type="password"
							id="password"
							name="password"
							required
							minlength="8"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="••••••••"
						/>
						<p class="text-xs text-gray-500 mt-1">Walker can change this after first login</p>
					</div>
				</div>

				<div class="mt-6 flex gap-3">
					<button
						type="button"
						on:click={() => (showCreateModal = false)}
						class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={isSubmitting}
						class="flex-1 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium disabled:opacity-50"
					>
						{isSubmitting ? 'Creating...' : 'Create Walker'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<!-- Schedule Modal -->
{#if showScheduleModal && selectedWalker}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-xl max-w-lg w-full p-6 max-h-[90vh] overflow-y-auto">
			<div class="flex items-center justify-between mb-6">
				<div>
					<h2 class="text-xl font-bold text-gray-900">Set Working Hours</h2>
					<p class="text-sm text-gray-500">{selectedWalker.first_name} {selectedWalker.last_name}</p>
				</div>
				<button on:click={closeScheduleModal} class="text-gray-400 hover:text-gray-600">
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<form
				method="POST"
				action="?/updateWorkingHours"
				use:enhance={() => {
					isSubmitting = true;
					return async ({ update }) => {
						isSubmitting = false;
						await update();
					};
				}}
			>
				<input type="hidden" name="walker_id" value={selectedWalker.id} />
				<input type="hidden" name="schedule" value={JSON.stringify(editSchedule)} />

				<div class="space-y-3">
					{#each dayNames as day, i}
						<div class="flex items-center gap-4 p-3 bg-gray-50 rounded-lg">
							<label class="flex items-center min-w-[100px]">
								<input
									type="checkbox"
									bind:checked={editSchedule[i].is_active}
									class="w-4 h-4 text-purple-600 border-gray-300 rounded focus:ring-purple-500"
								/>
								<span class="ml-2 text-sm font-medium text-gray-700">{day}</span>
							</label>

							<div class="flex items-center gap-2 flex-1">
								<input
									type="time"
									bind:value={editSchedule[i].start_time}
									disabled={!editSchedule[i].is_active}
									class="px-2 py-1 border border-gray-300 rounded text-sm disabled:bg-gray-100 disabled:text-gray-400"
								/>
								<span class="text-gray-400">to</span>
								<input
									type="time"
									bind:value={editSchedule[i].end_time}
									disabled={!editSchedule[i].is_active}
									class="px-2 py-1 border border-gray-300 rounded text-sm disabled:bg-gray-100 disabled:text-gray-400"
								/>
							</div>
						</div>
					{/each}
				</div>

				<div class="mt-6 flex gap-3">
					<button
						type="button"
						on:click={closeScheduleModal}
						class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={isSubmitting}
						class="flex-1 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium disabled:opacity-50"
					>
						{isSubmitting ? 'Saving...' : 'Save Schedule'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
