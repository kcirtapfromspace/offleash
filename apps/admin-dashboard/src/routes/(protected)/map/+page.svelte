<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import type { PageData } from './$types';
	import L from 'leaflet';

	export let data: PageData;

	let mapContainer: HTMLDivElement;
	let map: L.Map | null = null;
	let markers: L.Marker[] = [];
	let routeLine: L.Polyline | null = null;
	let selectedBookingId: string | null = null;

	// Status colors matching iOS
	const statusColors: Record<string, string> = {
		pending: '#f97316', // orange
		confirmed: '#3b82f6', // blue
		in_progress: '#8b5cf6', // purple
		completed: '#22c55e', // green
		cancelled: '#ef4444' // red
	};

	function getStatusColor(status: string): string {
		return statusColors[status] || '#6b7280';
	}

	function formatTime(isoString: string): string {
		return new Date(isoString).toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		});
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr + 'T12:00:00').toLocaleDateString('en-US', {
			weekday: 'long',
			month: 'long',
			day: 'numeric'
		});
	}

	function changeDate(offset: number) {
		const current = new Date(data.date + 'T12:00:00');
		current.setDate(current.getDate() + offset);
		const newDate = current.toISOString().split('T')[0];
		goto(`/map?date=${newDate}`);
	}

	function goToToday() {
		const today = new Date().toISOString().split('T')[0];
		goto(`/map?date=${today}`);
	}

	function createMarkerIcon(status: string, sequence?: number): L.DivIcon {
		const color = getStatusColor(status);
		const showSequence = sequence !== undefined && data.route?.is_optimized;

		return L.divIcon({
			className: 'custom-marker',
			html: `
				<div style="
					background-color: ${color};
					width: 32px;
					height: 32px;
					border-radius: 50% 50% 50% 0;
					transform: rotate(-45deg);
					border: 3px solid white;
					box-shadow: 0 2px 6px rgba(0,0,0,0.3);
					display: flex;
					align-items: center;
					justify-content: center;
				">
					${showSequence ? `
						<span style="
							transform: rotate(45deg);
							color: white;
							font-weight: bold;
							font-size: 12px;
						">${sequence}</span>
					` : ''}
				</div>
			`,
			iconSize: [32, 32],
			iconAnchor: [16, 32],
			popupAnchor: [0, -32]
		});
	}

	function initMap() {
		if (!mapContainer || map) return;

		// Default to Denver, Colorado
		const defaultCenter: [number, number] = [39.7392, -104.9903];
		const defaultZoom = 11;

		map = L.map(mapContainer).setView(defaultCenter, defaultZoom);

		// Add OpenStreetMap tiles
		L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
			attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
		}).addTo(map);

		updateMarkers();
	}

	function updateMarkers() {
		if (!map) return;

		// Clear existing markers
		markers.forEach((m) => m.remove());
		markers = [];
		if (routeLine) {
			routeLine.remove();
			routeLine = null;
		}

		if (!data.bookings || data.bookings.length === 0) return;

		const bounds = L.latLngBounds([]);

		// Create route sequence map if optimized
		const sequenceMap = new Map<string, number>();
		if (data.route?.is_optimized) {
			data.route.stops.forEach((stop) => {
				sequenceMap.set(stop.booking_id, stop.sequence);
			});
		}

		// Add markers for each booking
		data.bookings.forEach((booking) => {
			if (!booking.latitude || !booking.longitude) return;

			const sequence = sequenceMap.get(booking.id);
			const marker = L.marker([booking.latitude, booking.longitude], {
				icon: createMarkerIcon(booking.status, sequence)
			});

			// Create popup content with action buttons
			const navigateUrl = `https://maps.apple.com/?daddr=${encodeURIComponent(booking.address)}`;
			const callUrl = booking.customer_phone ? `tel:${booking.customer_phone}` : null;

			const popupContent = `
				<div style="min-width: 220px;">
					<div style="font-weight: bold; margin-bottom: 4px;">${booking.customer_name}</div>
					<div style="color: #666; font-size: 12px; margin-bottom: 4px;">${booking.service_name}</div>
					<div style="font-size: 12px; margin-bottom: 4px;">
						<strong>Time:</strong> ${formatTime(booking.scheduled_start)}
					</div>
					<div style="font-size: 12px; margin-bottom: 4px;">
						<strong>Address:</strong> ${booking.address}
					</div>
					<div style="margin-top: 8px;">
						<span style="
							display: inline-block;
							padding: 2px 8px;
							border-radius: 4px;
							font-size: 11px;
							font-weight: 500;
							background-color: ${getStatusColor(booking.status)}20;
							color: ${getStatusColor(booking.status)};
						">${booking.status.replace('_', ' ').toUpperCase()}</span>
					</div>
					${booking.notes ? `
						<div style="font-size: 12px; margin-top: 8px; padding-top: 8px; border-top: 1px solid #eee;">
							<strong>Notes:</strong> ${booking.notes}
						</div>
					` : ''}
					<div style="display: flex; gap: 8px; margin-top: 12px; padding-top: 12px; border-top: 1px solid #eee;">
						<a href="${navigateUrl}" target="_blank" style="
							flex: 1;
							display: flex;
							align-items: center;
							justify-content: center;
							gap: 4px;
							padding: 8px;
							background: #3b82f6;
							color: white;
							border-radius: 6px;
							text-decoration: none;
							font-size: 12px;
							font-weight: 500;
						">
							<svg width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
							</svg>
							Navigate
						</a>
						${callUrl ? `
							<a href="${callUrl}" style="
								flex: 1;
								display: flex;
								align-items: center;
								justify-content: center;
								gap: 4px;
								padding: 8px;
								background: #22c55e;
								color: white;
								border-radius: 6px;
								text-decoration: none;
								font-size: 12px;
								font-weight: 500;
							">
								<svg width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z"/>
								</svg>
								Call
							</a>
						` : ''}
					</div>
				</div>
			`;

			marker.bindPopup(popupContent);
			marker.on('click', () => {
				selectedBookingId = booking.id;
			});
			marker.addTo(map!);
			markers.push(marker);
			bounds.extend([booking.latitude, booking.longitude]);
		});

		// Draw route polyline if optimized
		if (data.route?.is_optimized && data.route.stops.length > 1) {
			const routeCoords: [number, number][] = [];

			// Build coordinates in sequence order
			data.route.stops.forEach((stop) => {
				const booking = data.bookings.find((b) => b.id === stop.booking_id);
				if (booking?.latitude && booking?.longitude) {
					routeCoords.push([booking.latitude, booking.longitude]);
				}
			});

			if (routeCoords.length > 1) {
				routeLine = L.polyline(routeCoords, {
					color: '#3b82f6',
					weight: 4,
					opacity: 0.7,
					dashArray: '10, 10'
				}).addTo(map);
			}
		}

		// Fit map to bounds
		if (bounds.isValid()) {
			map.fitBounds(bounds, { padding: [50, 50] });
		}
	}

	onMount(() => {
		// Dynamically import Leaflet CSS
		const link = document.createElement('link');
		link.rel = 'stylesheet';
		link.href = 'https://unpkg.com/leaflet@1.9.4/dist/leaflet.css';
		document.head.appendChild(link);

		// Wait for CSS to load then init map
		link.onload = () => {
			initMap();
		};
	});

	onDestroy(() => {
		if (map) {
			map.remove();
			map = null;
		}
	});

	// Reactively update markers when data changes
	$: if (map && data.bookings) {
		updateMarkers();
	}
</script>

<svelte:head>
	<title>Route Map - OFFLEASH</title>
</svelte:head>

<div class="h-screen flex flex-col">
	<!-- Header -->
	<div class="bg-white border-b border-gray-200 px-4 py-3">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-4">
				<h1 class="text-xl font-bold text-gray-900">Route Map</h1>
				{#if data.walkerName}
					<span class="text-sm text-gray-500">for {data.walkerName}</span>
				{/if}
			</div>

			<!-- Date Navigation -->
			<div class="flex items-center gap-2">
				<button
					on:click={() => changeDate(-1)}
					class="p-2 hover:bg-gray-100 rounded-lg"
					aria-label="Previous day"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
					</svg>
				</button>
				<button
					on:click={goToToday}
					class="px-3 py-1 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
				>
					Today
				</button>
				<span class="text-sm font-medium text-gray-900 min-w-[180px] text-center">
					{formatDate(data.date)}
				</span>
				<button
					on:click={() => changeDate(1)}
					class="p-2 hover:bg-gray-100 rounded-lg"
					aria-label="Next day"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
					</svg>
				</button>
			</div>
		</div>
	</div>

	<!-- Error State -->
	{#if data.error}
		<div class="flex-1 flex items-center justify-center bg-gray-50">
			<div class="text-center p-8">
				<svg class="w-16 h-16 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
				</svg>
				<p class="text-gray-600">{data.error}</p>
			</div>
		</div>
	{:else}
		<!-- Map and Summary -->
		<div class="flex-1 relative">
			<div bind:this={mapContainer} class="w-full h-full" />

			<!-- Summary Card (bottom overlay) -->
			{#if data.bookings.length > 0}
				<div class="absolute bottom-4 left-4 right-4 md:left-auto md:right-4 md:w-80 bg-white rounded-xl shadow-lg border border-gray-200 p-4">
					<!-- Stats -->
					<div class="grid grid-cols-3 gap-4 mb-3">
						<div class="text-center">
							<div class="text-2xl font-bold text-blue-600">{data.bookings.length}</div>
							<div class="text-xs text-gray-500">Stops</div>
						</div>
						<div class="text-center">
							<div class="text-2xl font-bold text-green-600">
								{data.route?.total_travel_minutes || 0}m
							</div>
							<div class="text-xs text-gray-500">Drive Time</div>
						</div>
						<div class="text-center">
							<div class="text-2xl font-bold text-orange-600">
								{Math.round((data.route?.total_distance_meters || 0) / 1609)}mi
							</div>
							<div class="text-xs text-gray-500">Distance</div>
						</div>
					</div>

					<!-- Optimization Status -->
					{#if data.route?.is_optimized}
						<div class="flex items-center gap-2 text-sm text-green-600 bg-green-50 px-3 py-2 rounded-lg">
							<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
								<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
							</svg>
							<span>Route optimized - saves {data.route.savings_minutes} min</span>
						</div>
					{:else if data.bookings.length > 1}
						<div class="flex items-center gap-2 text-sm text-gray-500 bg-gray-50 px-3 py-2 rounded-lg">
							<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
							<span>Route shown in scheduled order</span>
						</div>
					{/if}

					<!-- Legend -->
					<div class="mt-3 pt-3 border-t border-gray-100">
						<div class="text-xs text-gray-500 mb-2">Status Legend</div>
						<div class="flex flex-wrap gap-2">
							{#each Object.entries(statusColors) as [status, color]}
								<div class="flex items-center gap-1">
									<div class="w-3 h-3 rounded-full" style="background-color: {color}" />
									<span class="text-xs text-gray-600 capitalize">{status.replace('_', ' ')}</span>
								</div>
							{/each}
						</div>
					</div>
				</div>
			{/if}

			<!-- Empty State -->
			{#if data.bookings.length === 0 && !data.error}
				<div class="absolute inset-0 flex items-center justify-center bg-gray-50/80">
					<div class="text-center p-8 bg-white rounded-xl shadow-lg">
						<svg class="w-16 h-16 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
						</svg>
						<p class="text-lg font-medium text-gray-900 mb-1">No walks scheduled</p>
						<p class="text-gray-500">No active bookings for {formatDate(data.date)}</p>
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	:global(.custom-marker) {
		background: transparent;
		border: none;
	}

	:global(.leaflet-popup-content-wrapper) {
		border-radius: 8px;
	}

	:global(.leaflet-popup-content) {
		margin: 12px;
	}
</style>
