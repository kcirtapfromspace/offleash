<script lang="ts">
	import { page } from '$app/state';

	// Booking detail page
	const bookingId = page.params.id;
</script>

<div>
	<h1 class="text-2xl font-bold mb-6">Booking Details</h1>
	<p class="text-gray-600">Booking ID: {bookingId}</p>
</div>
