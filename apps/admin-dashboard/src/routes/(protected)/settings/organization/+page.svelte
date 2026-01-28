<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let showDeleteConfirm = $state(false);
	let confirmText = $state('');
	let deleting = $state(false);

	// The text user must type to confirm deletion
	const requiredConfirmText = 'DELETE';

	const canDelete = $derived(confirmText === requiredConfirmText);
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
		<h1 class="text-2xl font-bold mt-2">Organization Settings</h1>
		<p class="text-gray-500 text-sm">Manage your organization</p>
	</div>

	<!-- Error message -->
	{#if form?.error}
		<div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
			{form.error}
		</div>
	{/if}

	<!-- Organization Info -->
	<div class="bg-white rounded-lg shadow p-6 mb-6">
		<h2 class="text-lg font-semibold mb-4">Organization Information</h2>

		<dl class="space-y-4">
			<div>
				<dt class="text-sm font-medium text-gray-500">Organization Name</dt>
				<dd class="mt-1 text-lg text-gray-900">{data.organizationName}</dd>
			</div>

			{#if data.organizationSlug}
				<div>
					<dt class="text-sm font-medium text-gray-500">Organization Slug</dt>
					<dd class="mt-1 text-gray-900 font-mono">{data.organizationSlug}</dd>
				</div>
			{/if}
		</dl>
	</div>

	<!-- Danger Zone -->
	<div class="bg-white rounded-lg shadow border-2 border-red-200">
		<div class="p-6 border-b border-red-200 bg-red-50">
			<h2 class="text-lg font-semibold text-red-800">Danger Zone</h2>
			<p class="text-sm text-red-600 mt-1">
				Actions in this section are destructive and cannot be undone.
			</p>
		</div>

		<div class="p-6">
			<div class="flex items-start justify-between">
				<div>
					<h3 class="font-medium text-gray-900">Delete Organization</h3>
					<p class="text-sm text-gray-500 mt-1">
						Permanently delete this organization and all its data. This action cannot be undone.
					</p>
				</div>
				<button
					type="button"
					onclick={() => (showDeleteConfirm = true)}
					class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
				>
					Delete Organization
				</button>
			</div>
		</div>
	</div>
</div>

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-lg shadow-xl max-w-md w-full">
			<div class="p-6 border-b">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-full bg-red-100 flex items-center justify-center">
						<svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
							/>
						</svg>
					</div>
					<div>
						<h3 class="text-lg font-semibold text-gray-900">Delete Organization</h3>
						<p class="text-sm text-gray-500">This action is permanent</p>
					</div>
				</div>
			</div>

			<div class="p-6">
				<p class="text-gray-600 mb-4">
					You are about to permanently delete <strong>{data.organizationName}</strong>. This will:
				</p>

				<ul class="list-disc list-inside text-sm text-gray-600 mb-4 space-y-1">
					<li>Remove all organization data</li>
					<li>Cancel all active bookings</li>
					<li>Remove all team members' access</li>
					<li>Delete all customer records</li>
				</ul>

				<p class="text-sm text-gray-600 mb-4">
					To confirm, type <strong class="font-mono">{requiredConfirmText}</strong> below:
				</p>

				<input
					type="text"
					bind:value={confirmText}
					placeholder="Type DELETE to confirm"
					class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500 focus:border-red-500 mb-4"
				/>

				<div class="flex gap-3">
					<button
						type="button"
						onclick={() => {
							showDeleteConfirm = false;
							confirmText = '';
						}}
						class="flex-1 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
					>
						Cancel
					</button>

					<form
						method="POST"
						action="?/delete"
						class="flex-1"
						use:enhance={() => {
							deleting = true;
							return async ({ update }) => {
								await update();
								deleting = false;
							};
						}}
					>
						<button
							type="submit"
							disabled={!canDelete || deleting}
							class="w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2"
						>
							{#if deleting}
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
								Deleting...
							{:else}
								Delete Organization
							{/if}
						</button>
					</form>
				</div>
			</div>
		</div>
	</div>
{/if}
