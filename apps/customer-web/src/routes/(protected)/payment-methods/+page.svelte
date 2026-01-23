<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';

	let { data, form } = $props();

	let deleting = $state<string | null>(null);
	let settingDefault = $state<string | null>(null);
	let editingNickname = $state<string | null>(null);
	let nicknameValue = $state('');

	function getCardIcon(brand: string | null) {
		switch (brand?.toLowerCase()) {
			case 'visa':
				return 'Visa';
			case 'mastercard':
				return 'MC';
			case 'amex':
			case 'american express':
				return 'Amex';
			case 'discover':
				return 'Disc';
			default:
				return 'Card';
		}
	}

	function startEditNickname(methodId: string, currentNickname: string | null) {
		editingNickname = methodId;
		nicknameValue = currentNickname || '';
	}

	function cancelEditNickname() {
		editingNickname = null;
		nicknameValue = '';
	}
</script>

<div class="max-w-2xl mx-auto p-4">
	<div class="mb-6">
		<a href="/settings" class="text-sm text-gray-600 hover:text-gray-900 flex items-center gap-1">
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Settings
		</a>
	</div>

	<h1 class="text-2xl font-bold mb-2">Payment Methods</h1>
	<p class="text-gray-600 mb-6">Manage your saved payment methods for quick checkout.</p>

	{#if form?.error}
		<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
			{form.error}
		</div>
	{/if}

	{#if form?.success}
		<div class="mb-4 p-3 bg-green-100 border border-green-400 text-green-700 rounded">
			Payment method updated successfully.
		</div>
	{/if}

	<!-- Payment Methods List -->
	<div class="bg-white rounded-lg shadow divide-y">
		{#if data.paymentMethods.length === 0}
			<div class="p-8 text-center">
				<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 flex items-center justify-center">
					<svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
					</svg>
				</div>
				<h3 class="font-medium text-gray-900 mb-1">No payment methods</h3>
				<p class="text-sm text-gray-500 mb-4">
					Add a payment method when you book your first service.
				</p>
			</div>
		{:else}
			{#each data.paymentMethods as method}
				<div class="p-4">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-3">
							<!-- Card Icon -->
							<div class="w-12 h-8 rounded bg-gradient-to-br from-gray-700 to-gray-900 flex items-center justify-center text-white text-xs font-bold">
								{getCardIcon(method.card_brand)}
							</div>
							<div>
								{#if editingNickname === method.id}
									<form method="POST" action="?/updateNickname" use:enhance={() => {
										return async ({ update }) => {
											await update();
											editingNickname = null;
											await invalidateAll();
										};
									}}>
										<input type="hidden" name="methodId" value={method.id} />
										<div class="flex items-center gap-2">
											<input
												type="text"
												name="nickname"
												bind:value={nicknameValue}
												class="px-2 py-1 border border-gray-300 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
												placeholder="Card nickname"
											/>
											<button type="submit" class="text-sm text-blue-600 hover:text-blue-800">Save</button>
											<button type="button" onclick={cancelEditNickname} class="text-sm text-gray-600 hover:text-gray-800">Cancel</button>
										</div>
									</form>
								{:else}
									<p class="font-medium">
										{method.nickname || method.display_name}
										{#if method.is_default}
											<span class="ml-2 px-2 py-0.5 text-xs bg-green-100 text-green-700 rounded-full">Default</span>
										{/if}
										{#if method.is_expired}
											<span class="ml-2 px-2 py-0.5 text-xs bg-red-100 text-red-700 rounded-full">Expired</span>
										{/if}
									</p>
									{#if method.card_exp_month && method.card_exp_year}
										<p class="text-sm text-gray-500">
											Expires {method.card_exp_month.toString().padStart(2, '0')}/{method.card_exp_year}
										</p>
									{/if}
								{/if}
							</div>
						</div>

						{#if editingNickname !== method.id}
							<div class="flex items-center gap-2">
								<button
									type="button"
									onclick={() => startEditNickname(method.id, method.nickname)}
									class="text-sm text-gray-600 hover:text-gray-800"
								>
									Edit
								</button>

								{#if !method.is_default}
									<form method="POST" action="?/setDefault" use:enhance={() => {
										settingDefault = method.id;
										return async ({ update }) => {
											await update();
											settingDefault = null;
											await invalidateAll();
										};
									}}>
										<input type="hidden" name="methodId" value={method.id} />
										<button
											type="submit"
											disabled={settingDefault === method.id}
											class="text-sm text-blue-600 hover:text-blue-800 disabled:opacity-50"
										>
											{settingDefault === method.id ? 'Setting...' : 'Set Default'}
										</button>
									</form>
								{/if}

								<form method="POST" action="?/delete" use:enhance={() => {
									deleting = method.id;
									return async ({ update }) => {
										await update();
										deleting = null;
										await invalidateAll();
									};
								}}>
									<input type="hidden" name="methodId" value={method.id} />
									<button
										type="submit"
										disabled={deleting === method.id}
										class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
									>
										{deleting === method.id ? 'Deleting...' : 'Delete'}
									</button>
								</form>
							</div>
						{/if}
					</div>
				</div>
			{/each}
		{/if}
	</div>

	<div class="mt-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
		<p class="text-sm text-blue-800">
			<strong>Tip:</strong> Payment methods are automatically saved when you complete a booking.
			Your card details are securely stored by our payment processor.
		</p>
	</div>
</div>
