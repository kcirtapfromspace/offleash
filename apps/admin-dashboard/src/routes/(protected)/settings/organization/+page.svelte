<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Delete confirmation state - two-step process
	let deleteStep = $state(0); // 0: hidden, 1: first confirm, 2: final confirm
	let confirmText = $state('');
	let deleting = $state(false);
	const requiredConfirmText = 'DELETE';
	const canDelete = $derived(confirmText === requiredConfirmText);

	// Config saving state
	let savingConfig = $state(false);
	let showConfigSuccess = $state(false);

	// Payment form values
	let businessModel = $state(data.paymentConfig?.business_model ?? 'booking_only');
	let feeStructure = $state(data.paymentConfig?.fee_structure ?? 'customer_pays');
	let billingFrequency = $state(data.paymentConfig?.billing_frequency ?? 'monthly');
	let applePayEnabled = $state(data.paymentConfig?.apple_pay_enabled ?? false);
	let googlePayEnabled = $state(data.paymentConfig?.google_pay_enabled ?? false);
	let preferredProvider = $state(data.paymentConfig?.preferred_provider ?? '');

	// Check for connected providers
	const hasStripe = $derived(data.paymentProviders?.some((p) => p.provider_type === 'stripe'));
	const hasSquare = $derived(data.paymentProviders?.some((p) => p.provider_type === 'square'));

	// Handle OAuth redirects
	$effect(() => {
		if (form?.stripeConnectUrl) {
			window.location.href = form.stripeConnectUrl;
		}
		if (form?.squareConnectUrl) {
			window.location.href = form.squareConnectUrl;
		}
		if (form?.configSuccess) {
			showConfigSuccess = true;
			setTimeout(() => (showConfigSuccess = false), 3000);
		}
	});
</script>

<div class="p-6 max-w-3xl">
	<!-- Header -->
	<div class="mb-6">
		<a href="/settings" class="text-sm text-gray-500 hover:text-gray-700 flex items-center gap-1">
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Settings
		</a>
		<h1 class="text-2xl font-bold mt-2">Organization Settings</h1>
		<p class="text-gray-500 text-sm">Configure your business model and payment processing</p>
	</div>

	<!-- Success/Error Messages -->
	{#if showConfigSuccess}
		<div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg text-green-700">
			Configuration saved successfully!
		</div>
	{/if}

	{#if form?.configError}
		<div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
			{form.configError}
		</div>
	{/if}

	{#if form?.providerError}
		<div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
			{form.providerError}
		</div>
	{/if}

	<!-- Organization Info -->
	<div class="bg-white rounded-lg shadow p-6 mb-6">
		<h2 class="text-lg font-semibold mb-4">Organization Information</h2>
		<dl class="grid grid-cols-2 gap-4">
			<div>
				<dt class="text-sm font-medium text-gray-500">Name</dt>
				<dd class="mt-1 text-gray-900">{data.organizationName}</dd>
			</div>
			<div>
				<dt class="text-sm font-medium text-gray-500">Slug</dt>
				<dd class="mt-1 text-gray-900 font-mono">{data.organizationSlug}</dd>
			</div>
		</dl>
	</div>

	<!-- Business Model Configuration -->
	<div class="bg-white rounded-lg shadow p-6 mb-6">
		<h2 class="text-lg font-semibold mb-4">Business Model</h2>
		<p class="text-sm text-gray-600 mb-4">
			Choose how you want to use OFFLEASH and handle payments.
		</p>

		<form
			method="POST"
			action="?/updatePaymentConfig"
			use:enhance={() => {
				savingConfig = true;
				return async ({ update }) => {
					await update();
					savingConfig = false;
				};
			}}
		>
			<!-- Business Model Selection -->
			<div class="mb-6">
				<label class="block text-sm font-medium text-gray-700 mb-3">Service Type</label>
				<div class="space-y-3">
					<label
						class="flex items-start p-4 border rounded-lg cursor-pointer hover:bg-gray-50 {businessModel ===
						'booking_only'
							? 'border-blue-500 bg-blue-50'
							: 'border-gray-200'}"
					>
						<input
							type="radio"
							name="businessModel"
							value="booking_only"
							bind:group={businessModel}
							class="mt-1 mr-3"
						/>
						<div>
							<span class="font-medium">Booking Manager Only</span>
							<p class="text-sm text-gray-500 mt-1">
								Use OFFLEASH to manage bookings and schedules. You handle customer payments
								separately (cash, Venmo, etc.).
							</p>
						</div>
					</label>

					<label
						class="flex items-start p-4 border rounded-lg cursor-pointer hover:bg-gray-50 {businessModel ===
						'full_service'
							? 'border-blue-500 bg-blue-50'
							: 'border-gray-200'}"
					>
						<input
							type="radio"
							name="businessModel"
							value="full_service"
							bind:group={businessModel}
							class="mt-1 mr-3"
						/>
						<div>
							<span class="font-medium">Full Service with Payments</span>
							<p class="text-sm text-gray-500 mt-1">
								Process customer payments directly through the app. Accept credit cards, Apple Pay,
								and more.
							</p>
						</div>
					</label>
				</div>
			</div>

			<!-- Billing Frequency (for Booking Only) -->
			{#if businessModel === 'booking_only'}
				<div class="mb-6 p-4 bg-gray-50 rounded-lg">
					<label class="block text-sm font-medium text-gray-700 mb-2">Subscription Billing</label>
					<div class="flex gap-4">
						<label class="flex items-center">
							<input
								type="radio"
								name="billingFrequency"
								value="monthly"
								bind:group={billingFrequency}
								class="mr-2"
							/>
							<span>Monthly ($29/mo)</span>
						</label>
						<label class="flex items-center">
							<input
								type="radio"
								name="billingFrequency"
								value="yearly"
								bind:group={billingFrequency}
								class="mr-2"
							/>
							<span>Yearly ($290/yr - 2 months free)</span>
						</label>
					</div>
				</div>
			{/if}

			<!-- Fee Structure (for Full Service) -->
			{#if businessModel === 'full_service'}
				<div class="mb-6">
					<label class="block text-sm font-medium text-gray-700 mb-3">Fee Structure</label>
					<div class="space-y-3">
						<label
							class="flex items-start p-4 border rounded-lg cursor-pointer hover:bg-gray-50 {feeStructure ===
							'customer_pays'
								? 'border-blue-500 bg-blue-50'
								: 'border-gray-200'}"
						>
							<input
								type="radio"
								name="feeStructure"
								value="customer_pays"
								bind:group={feeStructure}
								class="mt-1 mr-3"
							/>
							<div>
								<span class="font-medium">Customer Pays Fees</span>
								<p class="text-sm text-gray-500 mt-1">
									Service fees (2.9% + $0.30) are added to customer's total. You receive the full
									service price.
								</p>
							</div>
						</label>

						<label
							class="flex items-start p-4 border rounded-lg cursor-pointer hover:bg-gray-50 {feeStructure ===
							'split_fees'
								? 'border-blue-500 bg-blue-50'
								: 'border-gray-200'}"
						>
							<input
								type="radio"
								name="feeStructure"
								value="split_fees"
								bind:group={feeStructure}
								class="mt-1 mr-3"
							/>
							<div>
								<span class="font-medium">Split Fees</span>
								<p class="text-sm text-gray-500 mt-1">
									Fees are split between you and your customers. Small subscription fee + reduced
									transaction fees.
								</p>
							</div>
						</label>

						<label
							class="flex items-start p-4 border rounded-lg cursor-pointer hover:bg-gray-50 {feeStructure ===
							'owner_subscription'
								? 'border-blue-500 bg-blue-50'
								: 'border-gray-200'}"
						>
							<input
								type="radio"
								name="feeStructure"
								value="owner_subscription"
								bind:group={feeStructure}
								class="mt-1 mr-3"
							/>
							<div>
								<span class="font-medium">Owner Subscription (No Customer Fees)</span>
								<p class="text-sm text-gray-500 mt-1">
									You pay a monthly subscription ($79/mo). Customers see no additional fees -
									cleaner checkout experience.
								</p>
							</div>
						</label>
					</div>
				</div>

				<!-- Payment Methods -->
				<div class="mb-6">
					<label class="block text-sm font-medium text-gray-700 mb-3">Accepted Payment Methods</label>
					<div class="space-y-2">
						<label class="flex items-center">
							<input type="checkbox" class="mr-2" checked disabled />
							<span>Credit/Debit Cards</span>
							<span class="ml-2 text-xs text-gray-500">(Always enabled)</span>
						</label>
						<label class="flex items-center">
							<input
								type="checkbox"
								name="applePayEnabled"
								bind:checked={applePayEnabled}
								class="mr-2"
							/>
							<span>Apple Pay</span>
						</label>
						<label class="flex items-center">
							<input
								type="checkbox"
								name="googlePayEnabled"
								bind:checked={googlePayEnabled}
								class="mr-2"
							/>
							<span>Google Pay</span>
						</label>
					</div>
				</div>
			{/if}

			<div class="flex justify-end">
				<button
					type="submit"
					disabled={savingConfig}
					class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
				>
					{savingConfig ? 'Saving...' : 'Save Configuration'}
				</button>
			</div>
		</form>
	</div>

	<!-- Payment Processor Connections -->
	{#if businessModel === 'full_service'}
		<div class="bg-white rounded-lg shadow p-6 mb-6">
			<h2 class="text-lg font-semibold mb-4">Payment Processors</h2>
			<p class="text-sm text-gray-600 mb-4">
				Connect your payment processor to start accepting payments from customers.
			</p>

			<div class="space-y-4">
				<!-- Stripe -->
				<div class="flex items-center justify-between p-4 border rounded-lg">
					<div class="flex items-center gap-3">
						<div class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
							<svg class="w-6 h-6 text-purple-600" viewBox="0 0 24 24" fill="currentColor">
								<path
									d="M13.976 9.15c-2.172-.806-3.356-1.426-3.356-2.409 0-.831.683-1.305 1.901-1.305 2.227 0 4.515.858 6.09 1.631l.89-5.494C18.252.975 15.697 0 12.165 0 9.667 0 7.589.654 6.104 1.872 4.56 3.147 3.757 4.992 3.757 7.218c0 4.039 2.467 5.76 6.476 7.219 2.585.92 3.445 1.574 3.445 2.583 0 .98-.84 1.545-2.354 1.545-1.875 0-4.965-.921-6.99-2.109l-.9 5.555C5.175 22.99 8.385 24 11.714 24c2.641 0 4.843-.624 6.328-1.813 1.664-1.305 2.525-3.236 2.525-5.732 0-4.128-2.524-5.851-6.591-7.305z"
								/>
							</svg>
						</div>
						<div>
							<p class="font-medium">Stripe</p>
							<p class="text-sm text-gray-500">
								{hasStripe ? 'Connected' : 'Accept credit cards, Apple Pay, Google Pay'}
							</p>
						</div>
					</div>
					{#if hasStripe}
						<form method="POST" action="?/disconnectProvider">
							<input
								type="hidden"
								name="providerId"
								value={data.paymentProviders?.find((p) => p.provider_type === 'stripe')?.id}
							/>
							<button
								type="submit"
								class="px-3 py-1 text-sm border border-red-300 text-red-600 rounded hover:bg-red-50"
							>
								Disconnect
							</button>
						</form>
					{:else}
						<form method="POST" action="?/connectStripe">
							<button
								type="submit"
								class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700"
							>
								Connect Stripe
							</button>
						</form>
					{/if}
				</div>

				<!-- Square -->
				<div class="flex items-center justify-between p-4 border rounded-lg">
					<div class="flex items-center gap-3">
						<div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
							<svg class="w-6 h-6 text-green-600" viewBox="0 0 24 24" fill="currentColor">
								<path
									d="M4.5 0A4.5 4.5 0 000 4.5v15A4.5 4.5 0 004.5 24h15a4.5 4.5 0 004.5-4.5v-15A4.5 4.5 0 0019.5 0h-15zm2.25 6.75h10.5A2.25 2.25 0 0119.5 9v6a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 15V9a2.25 2.25 0 012.25-2.25z"
								/>
							</svg>
						</div>
						<div>
							<p class="font-medium">Square</p>
							<p class="text-sm text-gray-500">
								{hasSquare ? 'Connected' : 'Accept payments with Square'}
							</p>
						</div>
					</div>
					{#if hasSquare}
						<form method="POST" action="?/disconnectProvider">
							<input
								type="hidden"
								name="providerId"
								value={data.paymentProviders?.find((p) => p.provider_type === 'square')?.id}
							/>
							<button
								type="submit"
								class="px-3 py-1 text-sm border border-red-300 text-red-600 rounded hover:bg-red-50"
							>
								Disconnect
							</button>
						</form>
					{:else}
						<form method="POST" action="?/connectSquare">
							<button
								type="submit"
								class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
							>
								Connect Square
							</button>
						</form>
					{/if}
				</div>
			</div>

			{#if hasStripe || hasSquare}
				<div class="mt-4">
					<label class="block text-sm font-medium text-gray-700 mb-2">Preferred Processor</label>
					<select
						bind:value={preferredProvider}
						class="w-full px-3 py-2 border border-gray-300 rounded-lg"
					>
						<option value="">Auto-select</option>
						{#if hasStripe}
							<option value="stripe">Stripe</option>
						{/if}
						{#if hasSquare}
							<option value="square">Square</option>
						{/if}
					</select>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Danger Zone -->
	<div class="bg-white rounded-lg shadow border-2 border-red-200">
		<div class="p-6 border-b border-red-200 bg-red-50">
			<h2 class="text-lg font-semibold text-red-800">Danger Zone</h2>
		</div>

		<div class="p-6">
			<div class="flex items-start justify-between">
				<div>
					<h3 class="font-medium text-gray-900">Delete Organization</h3>
					<p class="text-sm text-gray-500 mt-1">
						Permanently delete this organization. This cannot be undone.
					</p>
				</div>
				<button
					type="button"
					onclick={() => (deleteStep = 1)}
					class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
				>
					Delete Organization
				</button>
			</div>
		</div>
	</div>
</div>

<!-- Delete Confirmation Modal - Step 1 -->
{#if deleteStep === 1}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
			<div class="flex items-center gap-3 mb-4">
				<div class="w-10 h-10 rounded-full bg-yellow-100 flex items-center justify-center">
					<svg class="w-6 h-6 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
						/>
					</svg>
				</div>
				<h3 class="text-lg font-semibold">Are you sure?</h3>
			</div>

			<p class="text-gray-600 mb-4">
				You are about to delete <strong>{data.organizationName}</strong>. This will permanently
				remove:
			</p>

			<ul class="list-disc list-inside text-sm text-gray-600 mb-6 space-y-1">
				<li>All booking history and records</li>
				<li>Customer and walker accounts</li>
				<li>Payment processor connections</li>
				<li>All organization settings</li>
			</ul>

			<div class="flex gap-3">
				<button
					type="button"
					onclick={() => (deleteStep = 0)}
					class="flex-1 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
				>
					Cancel
				</button>
				<button
					type="button"
					onclick={() => (deleteStep = 2)}
					class="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
				>
					Yes, Continue
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Confirmation Modal - Step 2 (Final) -->
{#if deleteStep === 2}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
			<div class="flex items-center gap-3 mb-4">
				<div class="w-10 h-10 rounded-full bg-red-100 flex items-center justify-center">
					<svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
						/>
					</svg>
				</div>
				<div>
					<h3 class="text-lg font-semibold text-red-800">Final Confirmation</h3>
					<p class="text-sm text-gray-500">This action is irreversible</p>
				</div>
			</div>

			<p class="text-gray-600 mb-4">
				To confirm deletion, type <strong class="font-mono">{requiredConfirmText}</strong> below:
			</p>

			<input
				type="text"
				bind:value={confirmText}
				placeholder="Type DELETE to confirm"
				class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500 mb-4"
			/>

			{#if form?.deleteError}
				<div class="mb-4 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm">
					{form.deleteError}
				</div>
			{/if}

			<div class="flex gap-3">
				<button
					type="button"
					onclick={() => {
						deleteStep = 0;
						confirmText = '';
					}}
					class="flex-1 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
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
						class="w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{deleting ? 'Deleting...' : 'Delete Forever'}
					</button>
				</form>
			</div>
		</div>
	</div>
{/if}
