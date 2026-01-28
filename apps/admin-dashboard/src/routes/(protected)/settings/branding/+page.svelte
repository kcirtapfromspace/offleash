<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Font family options
	const fontFamilies = [
		{ value: '', label: 'Default (System)' },
		{ value: 'Inter', label: 'Inter' },
		{ value: 'Roboto', label: 'Roboto' },
		{ value: 'Open Sans', label: 'Open Sans' },
		{ value: 'Lato', label: 'Lato' },
		{ value: 'Poppins', label: 'Poppins' },
		{ value: 'Montserrat', label: 'Montserrat' }
	];

	let saving = $state(false);
	let showSuccess = $state(false);

	// Form values with defaults from current branding or form errors
	let primaryColor = $state(form?.primaryColor ?? data.branding?.primary_color ?? '#10B981');
	let secondaryColor = $state(form?.secondaryColor ?? data.branding?.secondary_color ?? '#3B82F6');
	let logoUrl = $state(form?.logoUrl ?? data.branding?.logo_url ?? '');
	let faviconUrl = $state(form?.faviconUrl ?? data.branding?.favicon_url ?? '');
	let fontFamily = $state(form?.fontFamily ?? data.branding?.font_family ?? '');

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
		<h1 class="text-2xl font-bold mt-2">Branding</h1>
		<p class="text-gray-500 text-sm">Customize your business appearance</p>
	</div>

	<!-- Success message -->
	{#if showSuccess}
		<div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg text-green-700">
			Branding updated successfully!
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
			<!-- Colors Section -->
			<div class="mb-8">
				<h2 class="text-lg font-semibold mb-4">Colors</h2>

				<div class="grid grid-cols-2 gap-4">
					<!-- Primary Color -->
					<div>
						<label for="primaryColor" class="block text-sm font-medium text-gray-700 mb-1">
							Primary Color
						</label>
						<div class="flex gap-2">
							<input
								type="color"
								id="primaryColorPicker"
								bind:value={primaryColor}
								class="h-10 w-14 rounded border border-gray-300 cursor-pointer"
							/>
							<input
								type="text"
								id="primaryColor"
								name="primaryColor"
								bind:value={primaryColor}
								placeholder="#10B981"
								pattern="^#[0-9A-Fa-f]{6}$"
								class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 uppercase"
							/>
						</div>
						<p class="mt-1 text-xs text-gray-500">Used for buttons and accents</p>
					</div>

					<!-- Secondary Color -->
					<div>
						<label for="secondaryColor" class="block text-sm font-medium text-gray-700 mb-1">
							Secondary Color
						</label>
						<div class="flex gap-2">
							<input
								type="color"
								id="secondaryColorPicker"
								bind:value={secondaryColor}
								class="h-10 w-14 rounded border border-gray-300 cursor-pointer"
							/>
							<input
								type="text"
								id="secondaryColor"
								name="secondaryColor"
								bind:value={secondaryColor}
								placeholder="#3B82F6"
								pattern="^#[0-9A-Fa-f]{6}$"
								class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 uppercase"
							/>
						</div>
						<p class="mt-1 text-xs text-gray-500">Used for links and highlights</p>
					</div>
				</div>
			</div>

			<!-- Logo Section -->
			<div class="mb-8">
				<h2 class="text-lg font-semibold mb-4">Logo</h2>

				<!-- Logo URL -->
				<div class="mb-4">
					<label for="logoUrl" class="block text-sm font-medium text-gray-700 mb-1">Logo URL</label>
					<input
						type="url"
						id="logoUrl"
						name="logoUrl"
						bind:value={logoUrl}
						placeholder="https://example.com/logo.png"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
					/>
					<p class="mt-1 text-xs text-gray-500">Recommended size: 200x50 pixels</p>
				</div>

				<!-- Logo Preview -->
				{#if logoUrl}
					<div class="mb-4 p-4 bg-gray-50 rounded-lg">
						<p class="text-sm text-gray-600 mb-2">Preview:</p>
						<img
							src={logoUrl}
							alt="Logo preview"
							class="max-h-12 object-contain"
							onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
						/>
					</div>
				{/if}

				<!-- Favicon URL -->
				<div>
					<label for="faviconUrl" class="block text-sm font-medium text-gray-700 mb-1">
						Favicon URL
					</label>
					<input
						type="url"
						id="faviconUrl"
						name="faviconUrl"
						bind:value={faviconUrl}
						placeholder="https://example.com/favicon.ico"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
					/>
					<p class="mt-1 text-xs text-gray-500">Browser tab icon (32x32 pixels)</p>
				</div>
			</div>

			<!-- Typography Section -->
			<div class="mb-8">
				<h2 class="text-lg font-semibold mb-4">Typography</h2>

				<div>
					<label for="fontFamily" class="block text-sm font-medium text-gray-700 mb-1">
						Font Family
					</label>
					<select
						id="fontFamily"
						name="fontFamily"
						bind:value={fontFamily}
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
					>
						{#each fontFamilies as font}
							<option value={font.value}>{font.label}</option>
						{/each}
					</select>
				</div>
			</div>

			<!-- Preview Section -->
			<div class="mb-8 p-4 border border-gray-200 rounded-lg">
				<h3 class="text-sm font-medium text-gray-700 mb-3">Preview</h3>
				<div class="flex items-center gap-4">
					<button
						type="button"
						class="px-4 py-2 rounded-lg text-white"
						style="background-color: {primaryColor}"
					>
						Primary Button
					</button>
					<a href="#preview" class="underline" style="color: {secondaryColor}">Link Example</a>
					<span style="font-family: {fontFamily || 'inherit'}">Sample Text</span>
				</div>
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
</div>
