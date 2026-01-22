<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';

	export let data: PageData;
	export let form: ActionData;

	const dayNames = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

	// Active tab
	let activeTab: 'profile' | 'schedule' | 'areas' = 'profile';

	// Profile form state
	let bio = data.profile?.bio || '';
	let profilePhotoUrl = data.profile?.profile_photo_url || '';
	let emergencyContactName = data.profile?.emergency_contact_name || '';
	let emergencyContactPhone = data.profile?.emergency_contact_phone || '';
	let emergencyContactRelationship = data.profile?.emergency_contact_relationship || '';
	let yearsExperience = data.profile?.years_experience || 0;
	let selectedSpecializations: string[] =
		data.profile?.specializations?.map((s) => s.specialization) || [];

	// Schedule state
	let editSchedule = dayNames.map((_, i) => {
		const existing = data.working_hours?.find((wh) => wh.day_of_week === i);
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

	// Service areas state
	let showAreaModal = false;
	let editingArea: (typeof data.service_areas)[0] | null = null;
	let areaName = '';
	let areaColor = '#3B82F6';
	let areaPolygon: { lat: number; lng: number }[] = [];
	let areaIsActive = true;
	let areaPriority = 0;
	let areaPriceAdjustment = 0;
	let areaNotes = '';
	let isDrawing = false;

	// Map state - Default to Denver
	let mapCenter = { lat: 39.7392, lng: -104.9903 };
	let mapZoom = 11;

	let isSubmitting = false;

	function toggleSpecialization(spec: string) {
		if (selectedSpecializations.includes(spec)) {
			selectedSpecializations = selectedSpecializations.filter((s) => s !== spec);
		} else {
			selectedSpecializations = [...selectedSpecializations, spec];
		}
	}

	function openCreateAreaModal() {
		editingArea = null;
		areaName = '';
		areaColor = '#3B82F6';
		areaPolygon = [];
		areaIsActive = true;
		areaPriority = data.service_areas.length;
		areaPriceAdjustment = 0;
		areaNotes = '';
		isDrawing = true;
		showAreaModal = true;
	}

	function openEditAreaModal(area: (typeof data.service_areas)[0]) {
		editingArea = area;
		areaName = area.name;
		areaColor = area.color;
		areaPolygon = [...area.polygon];
		areaIsActive = area.is_active;
		areaPriority = area.priority;
		areaPriceAdjustment = area.price_adjustment_percent;
		areaNotes = area.notes || '';
		isDrawing = false;
		showAreaModal = true;
	}

	function closeAreaModal() {
		showAreaModal = false;
		editingArea = null;
		isDrawing = false;
	}

	function addPolygonPoint(lat: number, lng: number) {
		if (isDrawing) {
			areaPolygon = [...areaPolygon, { lat, lng }];
		}
	}

	function removeLastPoint() {
		if (areaPolygon.length > 0) {
			areaPolygon = areaPolygon.slice(0, -1);
		}
	}

	function clearPolygon() {
		areaPolygon = [];
	}

	function getPolygonPath(): string {
		if (areaPolygon.length < 2) return '';
		return areaPolygon.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.lng},${p.lat}`).join(' ') + 'Z';
	}

	// Close modal on success
	$: if (form?.success) {
		showAreaModal = false;
		isSubmitting = false;
	}
</script>

<div class="p-6">
	<!-- Header -->
	<div class="mb-6">
		<a href="/walkers" class="text-purple-600 hover:text-purple-800 text-sm font-medium mb-2 inline-flex items-center">
			<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Walkers
		</a>
		<div class="flex items-center gap-4 mt-2">
			<div class="w-16 h-16 bg-purple-100 rounded-full flex items-center justify-center">
				<span class="text-purple-600 font-bold text-2xl">
					{data.walker.first_name[0]}{data.walker.last_name[0]}
				</span>
			</div>
			<div>
				<h1 class="text-2xl font-bold text-gray-900">
					{data.walker.first_name} {data.walker.last_name}
				</h1>
				<p class="text-gray-500">{data.walker.email}</p>
			</div>
		</div>
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

	<!-- Tabs -->
	<div class="border-b border-gray-200 mb-6">
		<nav class="-mb-px flex gap-8">
			<button
				onclick={() => (activeTab = 'profile')}
				class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {activeTab === 'profile'
					? 'border-purple-500 text-purple-600'
					: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
			>
				Profile
			</button>
			<button
				onclick={() => (activeTab = 'schedule')}
				class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {activeTab === 'schedule'
					? 'border-purple-500 text-purple-600'
					: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
			>
				Schedule
			</button>
			<button
				onclick={() => (activeTab = 'areas')}
				class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {activeTab === 'areas'
					? 'border-purple-500 text-purple-600'
					: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
			>
				Service Areas
			</button>
		</nav>
	</div>

	<!-- Profile Tab -->
	{#if activeTab === 'profile'}
		<form
			method="POST"
			action="?/updateProfile"
			use:enhance={() => {
				isSubmitting = true;
				return async ({ update }) => {
					isSubmitting = false;
					await update();
				};
			}}
			class="bg-white border border-gray-200 rounded-xl p-6"
		>
			<h2 class="text-lg font-semibold text-gray-900 mb-6">Walker Profile</h2>

			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				<!-- Bio -->
				<div class="md:col-span-2">
					<label for="bio" class="block text-sm font-medium text-gray-700 mb-1">Bio</label>
					<textarea
						id="bio"
						name="bio"
						bind:value={bio}
						rows="4"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
						placeholder="Tell customers about this walker's experience, approach, and personality..."
					></textarea>
				</div>

				<!-- Profile Photo URL -->
				<div class="md:col-span-2">
					<label for="profile_photo_url" class="block text-sm font-medium text-gray-700 mb-1">Profile Photo URL</label>
					<input
						type="url"
						id="profile_photo_url"
						name="profile_photo_url"
						bind:value={profilePhotoUrl}
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
						placeholder="https://example.com/photo.jpg"
					/>
				</div>

				<!-- Years Experience -->
				<div>
					<label for="years_experience" class="block text-sm font-medium text-gray-700 mb-1">Years of Experience</label>
					<input
						type="number"
						id="years_experience"
						name="years_experience"
						bind:value={yearsExperience}
						min="0"
						max="50"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
					/>
				</div>

				<!-- Spacer -->
				<div></div>

				<!-- Specializations -->
				<div class="md:col-span-2">
					<label class="block text-sm font-medium text-gray-700 mb-2">Specializations</label>
					<input type="hidden" name="specializations" value={JSON.stringify(selectedSpecializations)} />
					<div class="flex flex-wrap gap-2">
						{#each data.specialization_options as spec}
							<button
								type="button"
								onclick={() => toggleSpecialization(spec.value)}
								class="px-3 py-1.5 rounded-full text-sm font-medium transition-colors {selectedSpecializations.includes(spec.value)
									? 'bg-purple-600 text-white'
									: 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
							>
								{spec.display_name}
							</button>
						{/each}
					</div>
				</div>
			</div>

			<!-- Emergency Contact Section -->
			<div class="mt-8 pt-8 border-t border-gray-200">
				<h3 class="text-md font-semibold text-gray-900 mb-4">Emergency Contact</h3>
				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					<div>
						<label for="emergency_contact_name" class="block text-sm font-medium text-gray-700 mb-1">Name</label>
						<input
							type="text"
							id="emergency_contact_name"
							name="emergency_contact_name"
							bind:value={emergencyContactName}
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="Jane Doe"
						/>
					</div>
					<div>
						<label for="emergency_contact_phone" class="block text-sm font-medium text-gray-700 mb-1">Phone</label>
						<input
							type="tel"
							id="emergency_contact_phone"
							name="emergency_contact_phone"
							bind:value={emergencyContactPhone}
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="(555) 123-4567"
						/>
					</div>
					<div>
						<label for="emergency_contact_relationship" class="block text-sm font-medium text-gray-700 mb-1">Relationship</label>
						<input
							type="text"
							id="emergency_contact_relationship"
							name="emergency_contact_relationship"
							bind:value={emergencyContactRelationship}
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="Spouse, Parent, etc."
						/>
					</div>
				</div>
			</div>

			<div class="mt-6 flex justify-end">
				<button
					type="submit"
					disabled={isSubmitting}
					class="px-6 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium disabled:opacity-50"
				>
					{isSubmitting ? 'Saving...' : 'Save Profile'}
				</button>
			</div>
		</form>
	{/if}

	<!-- Schedule Tab -->
	{#if activeTab === 'schedule'}
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
			class="bg-white border border-gray-200 rounded-xl p-6"
		>
			<h2 class="text-lg font-semibold text-gray-900 mb-6">Working Hours</h2>
			<input type="hidden" name="schedule" value={JSON.stringify(editSchedule)} />

			<div class="space-y-3">
				{#each dayNames as day, i}
					<div class="flex items-center gap-4 p-3 bg-gray-50 rounded-lg">
						<label class="flex items-center min-w-[120px]">
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
								class="px-3 py-2 border border-gray-300 rounded-lg text-sm disabled:bg-gray-100 disabled:text-gray-400"
							/>
							<span class="text-gray-400">to</span>
							<input
								type="time"
								bind:value={editSchedule[i].end_time}
								disabled={!editSchedule[i].is_active}
								class="px-3 py-2 border border-gray-300 rounded-lg text-sm disabled:bg-gray-100 disabled:text-gray-400"
							/>
						</div>
					</div>
				{/each}
			</div>

			<div class="mt-6 flex justify-end">
				<button
					type="submit"
					disabled={isSubmitting}
					class="px-6 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium disabled:opacity-50"
				>
					{isSubmitting ? 'Saving...' : 'Save Schedule'}
				</button>
			</div>
		</form>
	{/if}

	<!-- Service Areas Tab -->
	{#if activeTab === 'areas'}
		<div class="bg-white border border-gray-200 rounded-xl p-6">
			<div class="flex items-center justify-between mb-6">
				<h2 class="text-lg font-semibold text-gray-900">Service Areas</h2>
				<button
					onclick={openCreateAreaModal}
					class="inline-flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium text-sm"
				>
					<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
					</svg>
					Add Area
				</button>
			</div>

			{#if data.service_areas.length === 0}
				<div class="text-center py-8">
					<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"
						/>
					</svg>
					<p class="text-gray-500">No service areas defined yet</p>
					<p class="text-sm text-gray-400 mt-1">Add service areas to define where this walker operates</p>
				</div>
			{:else}
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					{#each data.service_areas as area}
						<div class="border border-gray-200 rounded-lg p-4 hover:shadow-sm transition-shadow">
							<div class="flex items-start justify-between">
								<div class="flex items-center gap-3">
									<div
										class="w-4 h-4 rounded-full"
										style="background-color: {area.color}"
									></div>
									<div>
										<h3 class="font-medium text-gray-900">{area.name}</h3>
										<p class="text-sm text-gray-500">{area.polygon.length} points</p>
									</div>
								</div>
								<span
									class="px-2 py-0.5 text-xs font-medium rounded-full {area.is_active
										? 'bg-green-100 text-green-800'
										: 'bg-gray-100 text-gray-600'}"
								>
									{area.is_active ? 'Active' : 'Inactive'}
								</span>
							</div>

							{#if area.price_adjustment_percent !== 0}
								<p class="mt-2 text-sm text-gray-600">
									Price adjustment: {area.price_adjustment_percent > 0 ? '+' : ''}{area.price_adjustment_percent}%
								</p>
							{/if}

							{#if area.notes}
								<p class="mt-2 text-sm text-gray-500 truncate">{area.notes}</p>
							{/if}

							<div class="mt-3 pt-3 border-t border-gray-100 flex gap-4">
								<button
									onclick={() => openEditAreaModal(area)}
									class="text-sm font-medium text-purple-600 hover:text-purple-800"
								>
									Edit
								</button>
								<form method="POST" action="?/deleteServiceArea" use:enhance class="inline">
									<input type="hidden" name="area_id" value={area.id} />
									<button
										type="submit"
										class="text-sm font-medium text-red-600 hover:text-red-800"
										onclick={(e) => {
											if (!confirm('Are you sure you want to delete this service area?')) {
												e.preventDefault();
											}
										}}
									>
										Delete
									</button>
								</form>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>

<!-- Service Area Modal -->
{#if showAreaModal}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-xl max-w-4xl w-full p-6 max-h-[90vh] overflow-y-auto">
			<div class="flex items-center justify-between mb-6">
				<h2 class="text-xl font-bold text-gray-900">
					{editingArea ? 'Edit Service Area' : 'Create Service Area'}
				</h2>
				<button onclick={closeAreaModal} class="text-gray-400 hover:text-gray-600">
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<form
				method="POST"
				action={editingArea ? '?/updateServiceArea' : '?/createServiceArea'}
				use:enhance={() => {
					isSubmitting = true;
					return async ({ update }) => {
						isSubmitting = false;
						await update();
					};
				}}
			>
				{#if editingArea}
					<input type="hidden" name="area_id" value={editingArea.id} />
				{/if}
				<input type="hidden" name="polygon" value={JSON.stringify(areaPolygon)} />

				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<!-- Left column: Form fields -->
					<div class="space-y-4">
						<div>
							<label for="area_name" class="block text-sm font-medium text-gray-700 mb-1">Area Name</label>
							<input
								type="text"
								id="area_name"
								name="name"
								bind:value={areaName}
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
								placeholder="Downtown Denver"
							/>
						</div>

						<div>
							<label for="area_color" class="block text-sm font-medium text-gray-700 mb-1">Color</label>
							<div class="flex items-center gap-2">
								<input
									type="color"
									id="area_color"
									name="color"
									bind:value={areaColor}
									class="w-10 h-10 border border-gray-300 rounded cursor-pointer"
								/>
								<input
									type="text"
									bind:value={areaColor}
									class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500 font-mono text-sm"
								/>
							</div>
						</div>

						<div class="flex items-center gap-4">
							<label class="flex items-center">
								<input
									type="checkbox"
									name="is_active"
									bind:checked={areaIsActive}
									value="true"
									class="w-4 h-4 text-purple-600 border-gray-300 rounded focus:ring-purple-500"
								/>
								<span class="ml-2 text-sm font-medium text-gray-700">Active</span>
							</label>
						</div>

						<div class="grid grid-cols-2 gap-4">
							<div>
								<label for="area_priority" class="block text-sm font-medium text-gray-700 mb-1">Priority</label>
								<input
									type="number"
									id="area_priority"
									name="priority"
									bind:value={areaPriority}
									min="0"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
								/>
								<p class="text-xs text-gray-500 mt-1">Lower = higher priority</p>
							</div>
							<div>
								<label for="area_price_adj" class="block text-sm font-medium text-gray-700 mb-1">Price Adjustment %</label>
								<input
									type="number"
									id="area_price_adj"
									name="price_adjustment_percent"
									bind:value={areaPriceAdjustment}
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
								/>
							</div>
						</div>

						<div>
							<label for="area_notes" class="block text-sm font-medium text-gray-700 mb-1">Notes</label>
							<textarea
								id="area_notes"
								name="notes"
								bind:value={areaNotes}
								rows="2"
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
								placeholder="Optional notes about this area..."
							></textarea>
						</div>
					</div>

					<!-- Right column: Map -->
					<div>
						<div class="flex items-center justify-between mb-2">
							<label class="block text-sm font-medium text-gray-700">Service Area Boundary</label>
							<div class="flex gap-2">
								<button
									type="button"
									onclick={() => (isDrawing = !isDrawing)}
									class="px-2 py-1 text-xs font-medium rounded {isDrawing
										? 'bg-purple-600 text-white'
										: 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
								>
									{isDrawing ? 'Drawing...' : 'Draw'}
								</button>
								<button
									type="button"
									onclick={removeLastPoint}
									disabled={areaPolygon.length === 0}
									class="px-2 py-1 text-xs font-medium rounded bg-gray-100 text-gray-700 hover:bg-gray-200 disabled:opacity-50"
								>
									Undo
								</button>
								<button
									type="button"
									onclick={clearPolygon}
									disabled={areaPolygon.length === 0}
									class="px-2 py-1 text-xs font-medium rounded bg-red-100 text-red-700 hover:bg-red-200 disabled:opacity-50"
								>
									Clear
								</button>
							</div>
						</div>

						<!-- Map placeholder - in a real implementation, use a mapping library -->
						<div
							class="w-full h-64 bg-gray-100 rounded-lg border-2 border-dashed {isDrawing
								? 'border-purple-400 bg-purple-50'
								: 'border-gray-300'} relative overflow-hidden cursor-crosshair"
							onclick={(e) => {
								if (!isDrawing) return;
								const rect = e.currentTarget.getBoundingClientRect();
								const x = (e.clientX - rect.left) / rect.width;
								const y = (e.clientY - rect.top) / rect.height;
								// Convert to rough lat/lng for Denver area
								const lat = mapCenter.lat + (0.5 - y) * 0.2;
								const lng = mapCenter.lng + (x - 0.5) * 0.3;
								addPolygonPoint(lat, lng);
							}}
							onkeydown={() => {}}
							role="button"
							tabindex="0"
						>
							<!-- SVG overlay for polygon -->
							<svg class="absolute inset-0 w-full h-full pointer-events-none">
								{#if areaPolygon.length > 0}
									<!-- Draw polygon -->
									<polygon
										points={areaPolygon
											.map((p) => {
												const x = ((p.lng - mapCenter.lng) / 0.3 + 0.5) * 100;
												const y = (0.5 - (p.lat - mapCenter.lat) / 0.2) * 100;
												return `${x}%,${y}%`;
											})
											.join(' ')}
										fill={areaColor}
										fill-opacity="0.3"
										stroke={areaColor}
										stroke-width="2"
									/>
									<!-- Draw points -->
									{#each areaPolygon as point, i}
										{@const x = ((point.lng - mapCenter.lng) / 0.3 + 0.5) * 100}
										{@const y = (0.5 - (point.lat - mapCenter.lat) / 0.2) * 100}
										<circle cx="{x}%" cy="{y}%" r="4" fill={areaColor} stroke="white" stroke-width="2" />
									{/each}
								{/if}
							</svg>

							<!-- Instructions -->
							{#if areaPolygon.length === 0}
								<div class="absolute inset-0 flex items-center justify-center">
									<p class="text-gray-500 text-sm">
										{isDrawing ? 'Click to add boundary points' : 'Click "Draw" to start'}
									</p>
								</div>
							{/if}

							<!-- Point count -->
							{#if areaPolygon.length > 0}
								<div class="absolute bottom-2 left-2 bg-black bg-opacity-50 text-white text-xs px-2 py-1 rounded">
									{areaPolygon.length} point{areaPolygon.length !== 1 ? 's' : ''}
									{areaPolygon.length < 3 ? ' (need at least 3)' : ''}
								</div>
							{/if}
						</div>

						<p class="text-xs text-gray-500 mt-2">
							Click on the map to add boundary points. Need at least 3 points to create a valid area.
						</p>
					</div>
				</div>

				<div class="mt-6 flex gap-3 justify-end">
					<button
						type="button"
						onclick={closeAreaModal}
						class="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={isSubmitting || areaPolygon.length < 3 || !areaName}
						class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium disabled:opacity-50"
					>
						{isSubmitting ? 'Saving...' : editingArea ? 'Update Area' : 'Create Area'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
