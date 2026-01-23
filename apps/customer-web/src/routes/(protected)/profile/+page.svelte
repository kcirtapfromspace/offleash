<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';

	let { data, form } = $props();

	let showAddPetForm = $state(false);
	let editingPet = $state<string | null>(null);
	let deletingPet = $state<string | null>(null);
	let editingProfile = $state(false);

	// Form values for editing
	let firstName = $state(data.user?.first_name || '');
	let lastName = $state(data.user?.last_name || '');
	let phone = $state(data.user?.phone || '');

	function startEditProfile() {
		firstName = data.user?.first_name || '';
		lastName = data.user?.last_name || '';
		phone = data.user?.phone || '';
		editingProfile = true;
	}

	function cancelEditProfile() {
		editingProfile = false;
	}

	function startEditPet(petId: string) {
		editingPet = petId;
		showAddPetForm = false;
	}

	function cancelEditPet() {
		editingPet = null;
	}

	function startAddPet() {
		showAddPetForm = true;
		editingPet = null;
	}

	function cancelAddPet() {
		showAddPetForm = false;
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

	<h1 class="text-2xl font-bold mb-6">Profile</h1>

	{#if form?.error}
		<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
			{form.error}
		</div>
	{/if}

	{#if form?.success}
		<div class="mb-4 p-3 bg-green-100 border border-green-400 text-green-700 rounded">
			{form.message || 'Success!'}
		</div>
	{/if}

	<!-- Personal Information -->
	<div class="bg-white rounded-lg shadow p-6 mb-6">
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold">Personal Information</h2>
			{#if !editingProfile}
				<button
					type="button"
					onclick={startEditProfile}
					class="text-sm text-blue-600 hover:text-blue-800"
				>
					Edit
				</button>
			{/if}
		</div>

		{#if editingProfile}
			<form method="POST" action="?/updateProfile" use:enhance={() => {
				return async ({ update }) => {
					await update();
					editingProfile = false;
					await invalidateAll();
				};
			}}>
				<div class="space-y-4">
					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="first_name" class="block text-sm font-medium text-gray-700 mb-1">
								First Name
							</label>
							<input
								type="text"
								id="first_name"
								name="first_name"
								bind:value={firstName}
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
							/>
						</div>
						<div>
							<label for="last_name" class="block text-sm font-medium text-gray-700 mb-1">
								Last Name
							</label>
							<input
								type="text"
								id="last_name"
								name="last_name"
								bind:value={lastName}
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
							/>
						</div>
					</div>
					<div>
						<label for="phone" class="block text-sm font-medium text-gray-700 mb-1">
							Phone Number
						</label>
						<input
							type="tel"
							id="phone"
							name="phone"
							bind:value={phone}
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
							placeholder="(555) 123-4567"
						/>
					</div>
					<div class="flex gap-2">
						<button
							type="submit"
							class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
						>
							Save Changes
						</button>
						<button
							type="button"
							onclick={cancelEditProfile}
							class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
						>
							Cancel
						</button>
					</div>
				</div>
			</form>
		{:else}
			<div class="space-y-3">
				<div>
					<p class="text-sm text-gray-500">Name</p>
					<p class="font-medium">{data.user?.first_name} {data.user?.last_name}</p>
				</div>
				<div>
					<p class="text-sm text-gray-500">Email</p>
					<p class="font-medium">{data.user?.email}</p>
				</div>
				{#if data.user?.phone}
					<div>
						<p class="text-sm text-gray-500">Phone</p>
						<p class="font-medium">{data.user.phone}</p>
					</div>
				{/if}
			</div>
		{/if}
	</div>

	<!-- My Dogs Section -->
	<div class="bg-white rounded-lg shadow">
		<div class="p-6 border-b border-gray-200">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">My Dogs</h2>
				{#if !showAddPetForm}
					<button
						type="button"
						onclick={startAddPet}
						class="flex items-center gap-1 text-sm text-blue-600 hover:text-blue-800"
					>
						<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
						</svg>
						Add Dog
					</button>
				{/if}
			</div>
		</div>

		<!-- Add Pet Form -->
		{#if showAddPetForm}
			<div class="p-6 border-b border-gray-200 bg-gray-50">
				<h3 class="font-medium mb-4">Add New Dog</h3>
				<form method="POST" action="?/createPet" use:enhance={() => {
					return async ({ update }) => {
						await update();
						showAddPetForm = false;
						await invalidateAll();
					};
				}}>
					<div class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div>
								<label for="pet_name" class="block text-sm font-medium text-gray-700 mb-1">
									Name *
								</label>
								<input
									type="text"
									id="pet_name"
									name="name"
									required
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
									placeholder="Buddy"
								/>
							</div>
							<div>
								<label for="breed" class="block text-sm font-medium text-gray-700 mb-1">
									Breed
								</label>
								<input
									type="text"
									id="breed"
									name="breed"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
									placeholder="Golden Retriever"
								/>
							</div>
						</div>
						<div class="grid grid-cols-3 gap-4">
							<div>
								<label for="date_of_birth" class="block text-sm font-medium text-gray-700 mb-1">
									Birthday
								</label>
								<input
									type="date"
									id="date_of_birth"
									name="date_of_birth"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
								/>
							</div>
							<div>
								<label for="weight_lbs" class="block text-sm font-medium text-gray-700 mb-1">
									Weight (lbs)
								</label>
								<input
									type="number"
									id="weight_lbs"
									name="weight_lbs"
									step="0.1"
									min="0"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
									placeholder="45"
								/>
							</div>
							<div>
								<label for="gender" class="block text-sm font-medium text-gray-700 mb-1">
									Gender
								</label>
								<select
									id="gender"
									name="gender"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
								>
									<option value="">Select...</option>
									<option value="male">Male</option>
									<option value="female">Female</option>
								</select>
							</div>
						</div>
						<div class="grid grid-cols-2 gap-4">
							<div>
								<label for="color" class="block text-sm font-medium text-gray-700 mb-1">
									Color
								</label>
								<input
									type="text"
									id="color"
									name="color"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
									placeholder="Golden"
								/>
							</div>
							<div class="flex items-center">
								<label class="flex items-center gap-2 cursor-pointer mt-6">
									<input
										type="checkbox"
										name="is_spayed_neutered"
										value="true"
										class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
									/>
									<span class="text-sm text-gray-700">Spayed/Neutered</span>
								</label>
							</div>
						</div>
						<div>
							<label for="temperament" class="block text-sm font-medium text-gray-700 mb-1">
								Temperament
							</label>
							<input
								type="text"
								id="temperament"
								name="temperament"
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
								placeholder="Friendly, energetic, loves other dogs"
							/>
						</div>
						<div>
							<label for="special_needs" class="block text-sm font-medium text-gray-700 mb-1">
								Special Needs or Notes
							</label>
							<textarea
								id="special_needs"
								name="special_needs"
								rows="2"
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
								placeholder="Any allergies, medications, or special care instructions"
							></textarea>
						</div>
						<div class="grid grid-cols-2 gap-4">
							<div>
								<label for="vet_name" class="block text-sm font-medium text-gray-700 mb-1">
									Vet Name
								</label>
								<input
									type="text"
									id="vet_name"
									name="vet_name"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
									placeholder="Dr. Smith"
								/>
							</div>
							<div>
								<label for="vet_phone" class="block text-sm font-medium text-gray-700 mb-1">
									Vet Phone
								</label>
								<input
									type="tel"
									id="vet_phone"
									name="vet_phone"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
									placeholder="(555) 123-4567"
								/>
							</div>
						</div>
						<div class="flex gap-2">
							<button
								type="submit"
								class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
							>
								Add Dog
							</button>
							<button
								type="button"
								onclick={cancelAddPet}
								class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
							>
								Cancel
							</button>
						</div>
					</div>
				</form>
			</div>
		{/if}

		<!-- Pet List -->
		<div class="divide-y divide-gray-200">
			{#if data.pets.length === 0 && !showAddPetForm}
				<div class="p-8 text-center">
					<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 flex items-center justify-center">
						<svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
							<path stroke-linecap="round" stroke-linejoin="round" d="M12 9v6m3-3H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
					</div>
					<h3 class="font-medium text-gray-900 mb-1">No dogs added yet</h3>
					<p class="text-sm text-gray-500 mb-4">
						Add your furry friends to make booking walks easier.
					</p>
					<button
						type="button"
						onclick={startAddPet}
						class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
					>
						Add Your First Dog
					</button>
				</div>
			{:else}
				{#each data.pets as pet}
					<div class="p-4">
						{#if editingPet === pet.id}
							<!-- Edit Form -->
							<form method="POST" action="?/updatePet" use:enhance={() => {
								return async ({ update }) => {
									await update();
									editingPet = null;
									await invalidateAll();
								};
							}}>
								<input type="hidden" name="petId" value={pet.id} />
								<div class="space-y-4">
									<div class="grid grid-cols-2 gap-4">
										<div>
											<label class="block text-sm font-medium text-gray-700 mb-1">Name *</label>
											<input
												type="text"
												name="name"
												value={pet.name}
												required
												class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
											/>
										</div>
										<div>
											<label class="block text-sm font-medium text-gray-700 mb-1">Breed</label>
											<input
												type="text"
												name="breed"
												value={pet.breed || ''}
												class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
											/>
										</div>
									</div>
									<div class="grid grid-cols-3 gap-4">
										<div>
											<label class="block text-sm font-medium text-gray-700 mb-1">Birthday</label>
											<input
												type="date"
												name="date_of_birth"
												value={pet.date_of_birth || ''}
												class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
											/>
										</div>
										<div>
											<label class="block text-sm font-medium text-gray-700 mb-1">Weight (lbs)</label>
											<input
												type="number"
												name="weight_lbs"
												value={pet.weight_lbs || ''}
												step="0.1"
												min="0"
												class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
											/>
										</div>
										<div>
											<label class="block text-sm font-medium text-gray-700 mb-1">Gender</label>
											<select
												name="gender"
												class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
											>
												<option value="">Select...</option>
												<option value="male" selected={pet.gender === 'male'}>Male</option>
												<option value="female" selected={pet.gender === 'female'}>Female</option>
											</select>
										</div>
									</div>
									<div class="grid grid-cols-2 gap-4">
										<div>
											<label class="block text-sm font-medium text-gray-700 mb-1">Color</label>
											<input
												type="text"
												name="color"
												value={pet.color || ''}
												class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
											/>
										</div>
										<div class="flex items-center mt-6">
											<label class="flex items-center gap-2 cursor-pointer">
												<input
													type="checkbox"
													name="is_spayed_neutered"
													value="true"
													checked={pet.is_spayed_neutered}
													class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
												/>
												<span class="text-sm text-gray-700">Spayed/Neutered</span>
											</label>
										</div>
									</div>
									<div>
										<label class="block text-sm font-medium text-gray-700 mb-1">Temperament</label>
										<input
											type="text"
											name="temperament"
											value={pet.temperament || ''}
											class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
										/>
									</div>
									<div>
										<label class="block text-sm font-medium text-gray-700 mb-1">Special Needs</label>
										<textarea
											name="special_needs"
											rows="2"
											class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
										>{pet.special_needs || ''}</textarea>
									</div>
									<div class="flex gap-2">
										<button
											type="submit"
											class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
										>
											Save
										</button>
										<button
											type="button"
											onclick={cancelEditPet}
											class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
										>
											Cancel
										</button>
									</div>
								</div>
							</form>
						{:else}
							<!-- Display Mode -->
							<div class="flex items-start justify-between">
								<div class="flex items-center gap-4">
									<div class="w-16 h-16 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 flex items-center justify-center text-white text-2xl font-bold">
										{pet.name.charAt(0).toUpperCase()}
									</div>
									<div>
										<h3 class="font-semibold text-lg">{pet.name}</h3>
										<p class="text-sm text-gray-500">
											{pet.breed || 'Mixed breed'}
											{#if pet.age_years !== null}
												 &bull; {pet.age_years} year{pet.age_years === 1 ? '' : 's'} old
											{/if}
											{#if pet.weight_lbs}
												 &bull; {pet.weight_lbs} lbs
											{/if}
										</p>
										{#if pet.temperament}
											<p class="text-sm text-gray-600 mt-1">{pet.temperament}</p>
										{/if}
										{#if pet.special_needs}
											<p class="text-sm text-orange-600 mt-1">
												<strong>Special needs:</strong> {pet.special_needs}
											</p>
										{/if}
									</div>
								</div>
								<div class="flex items-center gap-2">
									<button
										type="button"
										onclick={() => startEditPet(pet.id)}
										class="text-sm text-blue-600 hover:text-blue-800"
									>
										Edit
									</button>
									<form method="POST" action="?/deletePet" use:enhance={() => {
										deletingPet = pet.id;
										return async ({ update }) => {
											await update();
											deletingPet = null;
											await invalidateAll();
										};
									}}>
										<input type="hidden" name="petId" value={pet.id} />
										<button
											type="submit"
											disabled={deletingPet === pet.id}
											class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
										>
											{deletingPet === pet.id ? 'Removing...' : 'Remove'}
										</button>
									</form>
								</div>
							</div>
						{/if}
					</div>
				{/each}
			{/if}
		</div>
	</div>
</div>
