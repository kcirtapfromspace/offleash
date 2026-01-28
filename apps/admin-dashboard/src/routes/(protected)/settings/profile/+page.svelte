<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Common timezones
	const timezones = [
		'America/New_York',
		'America/Chicago',
		'America/Denver',
		'America/Los_Angeles',
		'America/Phoenix',
		'America/Anchorage',
		'Pacific/Honolulu',
		'Europe/London',
		'Europe/Paris',
		'Asia/Tokyo',
		'Australia/Sydney'
	];

	let saving = $state(false);
	let showSuccess = $state(false);

	// Form values with defaults from current profile or form errors
	let firstName = $state(form?.firstName ?? data.profile?.first_name ?? '');
	let lastName = $state(form?.lastName ?? data.profile?.last_name ?? '');
	let phone = $state(form?.phone ?? data.profile?.phone ?? '');
	let timezone = $state(form?.timezone ?? data.profile?.timezone ?? 'America/Denver');

	// Reset success message after showing
	$effect(() => {
		if (form?.success) {
			showSuccess = true;
			setTimeout(() => {
				showSuccess = false;
			}, 3000);
		}
	});
</script>

<div class="p-6 max-w-2xl">
	<!-- Header with back link -->
	<div class="mb-6">
		<a href="/settings" class="text-sm text-gray-500 hover:text-gray-700 flex items-center gap-1">
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Settings
		</a>
		<h1 class="text-2xl font-bold mt-2">Profile</h1>
		<p class="text-gray-500 text-sm">Edit your personal information</p>
	</div>

	<!-- Success message -->
	{#if showSuccess}
		<div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg text-green-700">
			Profile updated successfully!
		</div>
	{/if}

	<!-- Error message -->
	{#if form?.error}
		<div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
			{form.error}
		</div>
	{/if}

	<div class="bg-white rounded-lg shadow p-6">
		<form
			method="POST"
			action="?/update"
			use:enhance={() => {
				saving = true;
				return async ({ update }) => {
					await update();
					saving = false;
				};
			}}
		>
			<!-- Email (readonly) -->
			<div class="mb-4">
				<label for="email" class="block text-sm font-medium text-gray-700 mb-1">Email</label>
				<input
					type="email"
					id="email"
					value={data.profile?.email ?? ''}
					disabled
					class="w-full px-3 py-2 border border-gray-300 rounded-lg bg-gray-50 text-gray-500 cursor-not-allowed"
				/>
				<p class="mt-1 text-xs text-gray-500">Email cannot be changed</p>
			</div>

			<!-- First Name -->
			<div class="mb-4">
				<label for="firstName" class="block text-sm font-medium text-gray-700 mb-1">
					First Name <span class="text-red-500">*</span>
				</label>
				<input
					type="text"
					id="firstName"
					name="firstName"
					bind:value={firstName}
					required
					class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
				/>
			</div>

			<!-- Last Name -->
			<div class="mb-4">
				<label for="lastName" class="block text-sm font-medium text-gray-700 mb-1">
					Last Name <span class="text-red-500">*</span>
				</label>
				<input
					type="text"
					id="lastName"
					name="lastName"
					bind:value={lastName}
					required
					class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
				/>
			</div>

			<!-- Phone -->
			<div class="mb-4">
				<label for="phone" class="block text-sm font-medium text-gray-700 mb-1">Phone</label>
				<input
					type="tel"
					id="phone"
					name="phone"
					bind:value={phone}
					placeholder="(555) 123-4567"
					class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
				/>
			</div>

			<!-- Timezone -->
			<div class="mb-6">
				<label for="timezone" class="block text-sm font-medium text-gray-700 mb-1">Timezone</label>
				<select
					id="timezone"
					name="timezone"
					bind:value={timezone}
					class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
				>
					{#each timezones as tz}
						<option value={tz}>{tz.replace('_', ' ')}</option>
					{/each}
				</select>
			</div>

			<!-- Submit Button -->
			<div class="flex justify-end">
				<button
					type="submit"
					disabled={saving}
					class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
				>
					{#if saving}
						<svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
							<circle
								class="opacity-25"
								cx="12"
								cy="12"
								r="10"
								stroke="currentColor"
								stroke-width="4"
							></circle>
							<path
								class="opacity-75"
								fill="currentColor"
								d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
							></path>
						</svg>
						Saving...
					{:else}
						Save Changes
					{/if}
				</button>
			</div>
		</form>
	</div>

	<!-- Account Info -->
	{#if data.profile}
		<div class="mt-6 bg-gray-50 rounded-lg p-4">
			<h3 class="text-sm font-medium text-gray-700 mb-2">Account Information</h3>
			<dl class="text-sm">
				<div class="flex justify-between py-1">
					<dt class="text-gray-500">Role</dt>
					<dd class="text-gray-900 capitalize">{data.profile.role}</dd>
				</div>
				<div class="flex justify-between py-1">
					<dt class="text-gray-500">Member since</dt>
					<dd class="text-gray-900">
						{new Date(data.profile.created_at).toLocaleDateString()}
					</dd>
				</div>
			</dl>
		</div>
	{/if}
</div>
