import { useEffect, useState, useCallback } from 'react';
import { get, post, ApiError } from '../lib/api';

interface Booking {
  id: string;
  customer_id: string;
  customer_name: string;
  walker_id: string;
  walker_name: string;
  service_id: string;
  service_name: string;
  location_id: string;
  location_address: string;
  status: string;
  scheduled_start: string;
  scheduled_end: string;
  price_cents: number;
  price_display: string;
  notes: string | null;
}

type StatusFilter =
  | 'all'
  | 'pending'
  | 'confirmed'
  | 'in_progress'
  | 'completed'
  | 'cancelled';

const STATUS_OPTIONS: { value: StatusFilter; label: string }[] = [
  { value: 'all', label: 'All Bookings' },
  { value: 'pending', label: 'Pending' },
  { value: 'confirmed', label: 'Confirmed' },
  { value: 'in_progress', label: 'In Progress' },
  { value: 'completed', label: 'Completed' },
  { value: 'cancelled', label: 'Cancelled' },
];

const STATUS_STYLES: Record<string, string> = {
  pending: 'bg-yellow-100 text-yellow-800',
  confirmed: 'bg-blue-100 text-blue-800',
  in_progress: 'bg-purple-100 text-purple-800',
  completed: 'bg-green-100 text-green-800',
  cancelled: 'bg-gray-100 text-gray-800',
  no_show: 'bg-red-100 text-red-800',
};

function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

function formatTime(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleTimeString('en-US', {
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  });
}

function formatStatusLabel(status: string): string {
  return status
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

export function BookingsPage() {
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all');
  const [selectedBooking, setSelectedBooking] = useState<Booking | null>(null);
  const [isActionLoading, setIsActionLoading] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const fetchBookings = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const url =
        statusFilter === 'all' ? '/bookings' : `/bookings?status=${statusFilter}`;
      const data = await get<Booking[]>(url);
      setBookings(data);
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.message || 'Failed to load bookings');
      } else {
        setError('An unexpected error occurred');
      }
    } finally {
      setIsLoading(false);
    }
  }, [statusFilter]);

  useEffect(() => {
    fetchBookings();
  }, [fetchBookings]);

  const handleRowClick = (booking: Booking) => {
    setSelectedBooking(booking);
    setActionError(null);
  };

  const closeModal = () => {
    setSelectedBooking(null);
    setActionError(null);
  };

  const handleConfirm = async () => {
    if (!selectedBooking) return;
    setIsActionLoading(true);
    setActionError(null);
    try {
      await post(`/bookings/${selectedBooking.id}/confirm`);
      closeModal();
      fetchBookings();
    } catch (err) {
      if (err instanceof ApiError) {
        setActionError(err.message || 'Failed to confirm booking');
      } else {
        setActionError('An unexpected error occurred');
      }
    } finally {
      setIsActionLoading(false);
    }
  };

  const handleCancel = async () => {
    if (!selectedBooking) return;
    setIsActionLoading(true);
    setActionError(null);
    try {
      await post(`/bookings/${selectedBooking.id}/cancel`);
      closeModal();
      fetchBookings();
    } catch (err) {
      if (err instanceof ApiError) {
        setActionError(err.message || 'Failed to cancel booking');
      } else {
        setActionError('An unexpected error occurred');
      }
    } finally {
      setIsActionLoading(false);
    }
  };

  const handleComplete = async () => {
    if (!selectedBooking) return;
    setIsActionLoading(true);
    setActionError(null);
    try {
      await post(`/bookings/${selectedBooking.id}/complete`);
      closeModal();
      fetchBookings();
    } catch (err) {
      if (err instanceof ApiError) {
        setActionError(err.message || 'Failed to complete booking');
      } else {
        setActionError('An unexpected error occurred');
      }
    } finally {
      setIsActionLoading(false);
    }
  };

  const canConfirm = selectedBooking?.status === 'pending';
  const canCancel =
    selectedBooking?.status === 'pending' ||
    selectedBooking?.status === 'confirmed';
  const canComplete =
    selectedBooking?.status === 'confirmed' ||
    selectedBooking?.status === 'in_progress';

  const modal = selectedBooking && (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex min-h-screen items-center justify-center p-4">
        <div
          className="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
          onClick={closeModal}
        ></div>

        <div className="relative bg-white rounded-lg shadow-xl w-full max-w-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold">Booking Details</h2>
            <button
              type="button"
              onClick={closeModal}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg
                className="w-6 h-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          {actionError && (
            <div className="mb-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded text-sm">
              {actionError}
            </div>
          )}

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-500">Status</span>
              <span
                className={`px-2 py-1 text-xs font-semibold rounded-full ${
                  STATUS_STYLES[selectedBooking.status] ||
                  'bg-gray-100 text-gray-800'
                }`}
              >
                {formatStatusLabel(selectedBooking.status)}
              </span>
            </div>

            <div className="border-t border-gray-200 pt-4">
              <h3 className="text-sm font-medium text-gray-900 mb-2">
                Customer
              </h3>
              <p className="text-sm text-gray-600">
                {selectedBooking.customer_name}
              </p>
            </div>

            <div className="border-t border-gray-200 pt-4">
              <h3 className="text-sm font-medium text-gray-900 mb-2">Walker</h3>
              <p className="text-sm text-gray-600">
                {selectedBooking.walker_name}
              </p>
            </div>

            <div className="border-t border-gray-200 pt-4">
              <h3 className="text-sm font-medium text-gray-900 mb-2">
                Service
              </h3>
              <p className="text-sm text-gray-600">
                {selectedBooking.service_name}
              </p>
            </div>

            <div className="border-t border-gray-200 pt-4">
              <h3 className="text-sm font-medium text-gray-900 mb-2">
                Location
              </h3>
              <p className="text-sm text-gray-600">
                {selectedBooking.location_address}
              </p>
            </div>

            <div className="border-t border-gray-200 pt-4">
              <h3 className="text-sm font-medium text-gray-900 mb-2">
                Schedule
              </h3>
              <div className="text-sm text-gray-600">
                <p>{formatDate(selectedBooking.scheduled_start)}</p>
                <p>
                  {formatTime(selectedBooking.scheduled_start)} -{' '}
                  {formatTime(selectedBooking.scheduled_end)}
                </p>
              </div>
            </div>

            <div className="border-t border-gray-200 pt-4">
              <h3 className="text-sm font-medium text-gray-900 mb-2">Price</h3>
              <p className="text-sm text-gray-600">
                {selectedBooking.price_display}
              </p>
            </div>

            {selectedBooking.notes && (
              <div className="border-t border-gray-200 pt-4">
                <h3 className="text-sm font-medium text-gray-900 mb-2">
                  Notes
                </h3>
                <p className="text-sm text-gray-600">{selectedBooking.notes}</p>
              </div>
            )}
          </div>

          <div className="mt-6 flex justify-end space-x-3">
            {canConfirm && (
              <button
                type="button"
                onClick={handleConfirm}
                disabled={isActionLoading}
                className="px-4 py-2 text-sm font-medium text-white bg-green-600 rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isActionLoading ? 'Processing...' : 'Confirm'}
              </button>
            )}
            {canComplete && (
              <button
                type="button"
                onClick={handleComplete}
                disabled={isActionLoading}
                className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isActionLoading ? 'Processing...' : 'Mark Complete'}
              </button>
            )}
            {canCancel && (
              <button
                type="button"
                onClick={handleCancel}
                disabled={isActionLoading}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isActionLoading ? 'Processing...' : 'Cancel'}
              </button>
            )}
            <button
              type="button"
              onClick={closeModal}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );

  const header = (
    <div className="flex items-center justify-between mb-6">
      <h1 className="text-2xl font-bold">Bookings</h1>
      <select
        value={statusFilter}
        onChange={(e) => setStatusFilter(e.target.value as StatusFilter)}
        className="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        {STATUS_OPTIONS.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );

  if (isLoading) {
    return (
      <div className="p-6">
        {header}
        <div className="bg-white rounded-lg shadow-md overflow-hidden">
          <div className="animate-pulse">
            <div className="h-12 bg-gray-100 border-b border-gray-200"></div>
            {[1, 2, 3, 4, 5].map((i) => (
              <div key={i} className="flex border-b border-gray-200 p-4">
                <div className="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
                <div className="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
                <div className="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
                <div className="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
                <div className="h-4 bg-gray-200 rounded w-1/12"></div>
              </div>
            ))}
          </div>
        </div>
        {modal}
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        {header}
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
        {modal}
      </div>
    );
  }

  if (bookings.length === 0) {
    return (
      <div className="p-6">
        {header}
        <div className="bg-white rounded-lg shadow-md p-8 text-center">
          <div className="text-gray-400 mb-4">
            <svg
              className="w-16 h-16 mx-auto"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
              />
            </svg>
          </div>
          <h2 className="text-xl font-semibold text-gray-700 mb-2">
            No bookings found
          </h2>
          <p className="text-gray-500">
            {statusFilter === 'all'
              ? 'There are no bookings yet.'
              : `There are no ${statusFilter.replace('_', ' ')} bookings.`}
          </p>
        </div>
        {modal}
      </div>
    );
  }

  return (
    <div className="p-6">
      {header}

      <div className="bg-white rounded-lg shadow-md overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Date/Time
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Customer
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Walker
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Service
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Status
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {bookings.map((booking) => (
              <tr
                key={booking.id}
                onClick={() => handleRowClick(booking)}
                className="hover:bg-gray-50 cursor-pointer"
              >
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm font-medium text-gray-900">
                    {formatDate(booking.scheduled_start)}
                  </div>
                  <div className="text-sm text-gray-500">
                    {formatTime(booking.scheduled_start)}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm text-gray-900">
                    {booking.customer_name}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm text-gray-900">
                    {booking.walker_name}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm text-gray-900">
                    {booking.service_name}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span
                    className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                      STATUS_STYLES[booking.status] ||
                      'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {formatStatusLabel(booking.status)}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {modal}
    </div>
  );
}
