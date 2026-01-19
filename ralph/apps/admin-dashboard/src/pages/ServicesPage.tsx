import { useEffect, useState, useCallback } from 'react';
import { get, post, patch, ApiError } from '../lib/api';

interface Service {
  id: string;
  name: string;
  description: string | null;
  duration_minutes: number;
  price_cents: number;
  price_display: string;
  is_active: boolean;
}

interface ServiceFormData {
  name: string;
  description: string;
  duration_minutes: string;
  base_price_cents: string;
  is_active: boolean;
}

interface FormErrors {
  name?: string;
  duration_minutes?: string;
  base_price_cents?: string;
  general?: string;
}

const initialFormData: ServiceFormData = {
  name: '',
  description: '',
  duration_minutes: '',
  base_price_cents: '',
  is_active: true,
};

export function ServicesPage() {
  const [services, setServices] = useState<Service[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [editingService, setEditingService] = useState<Service | null>(null);
  const [formData, setFormData] = useState<ServiceFormData>(initialFormData);
  const [formErrors, setFormErrors] = useState<FormErrors>({});

  const fetchServices = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const data = await get<Service[]>('/services');
      setServices(data);
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.message || 'Failed to load services');
      } else {
        setError('An unexpected error occurred');
      }
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchServices();
  }, [fetchServices]);

  const validateForm = (): boolean => {
    const errors: FormErrors = {};

    if (!formData.name.trim()) {
      errors.name = 'Name is required';
    }

    const duration = parseInt(formData.duration_minutes, 10);
    if (!formData.duration_minutes.trim() || isNaN(duration) || duration <= 0) {
      errors.duration_minutes = 'Duration must be a positive number';
    }

    const price = parseInt(formData.base_price_cents, 10);
    if (!formData.base_price_cents.trim() || isNaN(price) || price < 0) {
      errors.base_price_cents = 'Price must be a non-negative number';
    }

    setFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsSubmitting(true);
    setFormErrors({});

    try {
      const payload = {
        name: formData.name.trim(),
        description: formData.description.trim() || null,
        duration_minutes: parseInt(formData.duration_minutes, 10),
        base_price_cents: parseInt(formData.base_price_cents, 10),
        is_active: formData.is_active,
      };

      if (editingService) {
        await patch(`/services/${editingService.id}`, payload);
      } else {
        await post('/services', payload);
      }

      closeModal();
      fetchServices();
    } catch (err) {
      if (err instanceof ApiError) {
        setFormErrors({ general: err.message || 'Failed to save service' });
      } else {
        setFormErrors({ general: 'An unexpected error occurred' });
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleInputChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    const { name, value, type } = e.target;
    const checked =
      type === 'checkbox' ? (e.target as HTMLInputElement).checked : undefined;
    setFormData((prev) => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value,
    }));
    if (formErrors[name as keyof FormErrors]) {
      setFormErrors((prev) => ({ ...prev, [name]: undefined }));
    }
  };

  const openModal = (service?: Service) => {
    if (service) {
      setEditingService(service);
      setFormData({
        name: service.name,
        description: service.description || '',
        duration_minutes: service.duration_minutes.toString(),
        base_price_cents: service.price_cents.toString(),
        is_active: service.is_active,
      });
    } else {
      setEditingService(null);
      setFormData(initialFormData);
    }
    setIsModalOpen(true);
    setFormErrors({});
  };

  const closeModal = () => {
    setIsModalOpen(false);
    setEditingService(null);
    setFormData(initialFormData);
    setFormErrors({});
  };

  const formatPrice = (cents: number): string => {
    return `$${(cents / 100).toFixed(2)}`;
  };

  const modal = isModalOpen && (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex min-h-screen items-center justify-center p-4">
        <div
          className="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
          onClick={closeModal}
        ></div>

        <div className="relative bg-white rounded-lg shadow-xl w-full max-w-md p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold">
              {editingService ? 'Edit Service' : 'Add Service'}
            </h2>
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

          {formErrors.general && (
            <div className="mb-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded text-sm">
              {formErrors.general}
            </div>
          )}

          <form onSubmit={handleSubmit}>
            <div className="space-y-4">
              <div>
                <label
                  htmlFor="name"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Name *
                </label>
                <input
                  type="text"
                  id="name"
                  name="name"
                  value={formData.name}
                  onChange={handleInputChange}
                  className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    formErrors.name ? 'border-red-500' : 'border-gray-300'
                  }`}
                />
                {formErrors.name && (
                  <p className="mt-1 text-sm text-red-600">{formErrors.name}</p>
                )}
              </div>

              <div>
                <label
                  htmlFor="description"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Description
                </label>
                <textarea
                  id="description"
                  name="description"
                  value={formData.description}
                  onChange={handleInputChange}
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label
                  htmlFor="duration_minutes"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Duration (minutes) *
                </label>
                <input
                  type="number"
                  id="duration_minutes"
                  name="duration_minutes"
                  value={formData.duration_minutes}
                  onChange={handleInputChange}
                  min="1"
                  className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    formErrors.duration_minutes
                      ? 'border-red-500'
                      : 'border-gray-300'
                  }`}
                />
                {formErrors.duration_minutes && (
                  <p className="mt-1 text-sm text-red-600">
                    {formErrors.duration_minutes}
                  </p>
                )}
              </div>

              <div>
                <label
                  htmlFor="base_price_cents"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Price (cents) *
                </label>
                <input
                  type="number"
                  id="base_price_cents"
                  name="base_price_cents"
                  value={formData.base_price_cents}
                  onChange={handleInputChange}
                  min="0"
                  className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    formErrors.base_price_cents
                      ? 'border-red-500'
                      : 'border-gray-300'
                  }`}
                />
                {formErrors.base_price_cents && (
                  <p className="mt-1 text-sm text-red-600">
                    {formErrors.base_price_cents}
                  </p>
                )}
                {formData.base_price_cents && (
                  <p className="mt-1 text-sm text-gray-500">
                    Display price:{' '}
                    {formatPrice(parseInt(formData.base_price_cents, 10) || 0)}
                  </p>
                )}
              </div>

              {editingService && (
                <div className="flex items-center">
                  <input
                    type="checkbox"
                    id="is_active"
                    name="is_active"
                    checked={formData.is_active}
                    onChange={handleInputChange}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <label
                    htmlFor="is_active"
                    className="ml-2 block text-sm text-gray-700"
                  >
                    Active
                  </label>
                </div>
              )}
            </div>

            <div className="mt-6 flex justify-end space-x-3">
              <button
                type="button"
                onClick={closeModal}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={isSubmitting}
                className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSubmitting
                  ? 'Saving...'
                  : editingService
                    ? 'Update Service'
                    : 'Add Service'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );

  const header = (
    <div className="flex items-center justify-between mb-6">
      <h1 className="text-2xl font-bold">Services</h1>
      <button
        onClick={() => openModal()}
        className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        Add Service
      </button>
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
                <div className="h-4 bg-gray-200 rounded w-1/4 mr-4"></div>
                <div className="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
                <div className="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
                <div className="h-4 bg-gray-200 rounded w-1/6"></div>
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

  if (services.length === 0) {
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
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
          </div>
          <h2 className="text-xl font-semibold text-gray-700 mb-2">
            No services found
          </h2>
          <p className="text-gray-500 mb-4">
            There are no services configured for your organization yet.
          </p>
          <button
            onClick={() => openModal()}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            Add Your First Service
          </button>
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
                Name
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Duration
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Price
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Status
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {services.map((service) => (
              <tr key={service.id} className="hover:bg-gray-50">
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm font-medium text-gray-900">
                    {service.name}
                  </div>
                  {service.description && (
                    <div className="text-sm text-gray-500 truncate max-w-xs">
                      {service.description}
                    </div>
                  )}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm text-gray-900">
                    {service.duration_minutes} min
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm text-gray-900">
                    {service.price_display}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span
                    className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                      service.is_active
                        ? 'bg-green-100 text-green-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {service.is_active ? 'Active' : 'Inactive'}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm">
                  <button
                    onClick={() => openModal(service)}
                    className="text-blue-600 hover:text-blue-900"
                  >
                    Edit
                  </button>
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
