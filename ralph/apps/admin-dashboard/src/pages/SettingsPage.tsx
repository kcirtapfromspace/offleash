import { useEffect, useState, useCallback } from 'react';
import { get, put, ApiError } from '../lib/api';

interface Branding {
  company_name: string;
  primary_color: string;
  secondary_color: string;
  accent_color: string;
  support_email: string;
  support_phone: string;
}

interface FormErrors {
  company_name?: string;
  primary_color?: string;
  secondary_color?: string;
  accent_color?: string;
  support_email?: string;
  support_phone?: string;
  general?: string;
}

interface Toast {
  type: 'success' | 'error';
  message: string;
}

const initialFormData: Branding = {
  company_name: '',
  primary_color: '#3B82F6',
  secondary_color: '#6B7280',
  accent_color: '#10B981',
  support_email: '',
  support_phone: '',
};

export function SettingsPage() {
  const [formData, setFormData] = useState<Branding>(initialFormData);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formErrors, setFormErrors] = useState<FormErrors>({});
  const [toast, setToast] = useState<Toast | null>(null);

  const fetchBranding = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const data = await get<Branding>('/admin/branding');
      setFormData(data);
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.message || 'Failed to load branding settings');
      } else {
        setError('An unexpected error occurred');
      }
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBranding();
  }, [fetchBranding]);

  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => setToast(null), 5000);
      return () => clearTimeout(timer);
    }
  }, [toast]);

  const validateForm = (): boolean => {
    const errors: FormErrors = {};

    if (!formData.company_name.trim()) {
      errors.company_name = 'Company name is required';
    }

    const colorRegex = /^#[0-9A-Fa-f]{6}$/;
    if (!colorRegex.test(formData.primary_color)) {
      errors.primary_color = 'Invalid color format';
    }
    if (!colorRegex.test(formData.secondary_color)) {
      errors.secondary_color = 'Invalid color format';
    }
    if (!colorRegex.test(formData.accent_color)) {
      errors.accent_color = 'Invalid color format';
    }

    if (formData.support_email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.support_email)) {
      errors.support_email = 'Invalid email format';
    }

    setFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsSaving(true);
    setFormErrors({});

    try {
      const payload = {
        company_name: formData.company_name.trim(),
        primary_color: formData.primary_color,
        secondary_color: formData.secondary_color,
        accent_color: formData.accent_color,
        support_email: formData.support_email.trim() || null,
        support_phone: formData.support_phone.trim() || null,
      };

      await put('/admin/branding', payload);
      setToast({ type: 'success', message: 'Branding settings saved successfully' });
    } catch (err) {
      if (err instanceof ApiError) {
        setFormErrors({ general: err.message || 'Failed to save branding settings' });
        setToast({ type: 'error', message: err.message || 'Failed to save branding settings' });
      } else {
        setFormErrors({ general: 'An unexpected error occurred' });
        setToast({ type: 'error', message: 'An unexpected error occurred' });
      }
    } finally {
      setIsSaving(false);
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));
    if (formErrors[name as keyof FormErrors]) {
      setFormErrors((prev) => ({ ...prev, [name]: undefined }));
    }
  };

  if (isLoading) {
    return (
      <div className="p-6">
        <h1 className="text-2xl font-bold mb-6">Settings</h1>
        <div className="bg-white rounded-lg shadow-md p-6">
          <div className="animate-pulse space-y-4">
            <div className="h-8 bg-gray-200 rounded w-1/4"></div>
            <div className="space-y-3">
              {[1, 2, 3, 4, 5, 6].map((i) => (
                <div key={i} className="space-y-2">
                  <div className="h-4 bg-gray-200 rounded w-1/6"></div>
                  <div className="h-10 bg-gray-200 rounded w-full"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <h1 className="text-2xl font-bold mb-6">Settings</h1>
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Settings</h1>

      {toast && (
        <div
          className={`mb-4 px-4 py-3 rounded flex items-center justify-between ${
            toast.type === 'success'
              ? 'bg-green-100 border border-green-400 text-green-700'
              : 'bg-red-100 border border-red-400 text-red-700'
          }`}
        >
          <span>{toast.message}</span>
          <button
            onClick={() => setToast(null)}
            className="ml-4 text-lg font-semibold hover:opacity-70"
          >
            ×
          </button>
        </div>
      )}

      <div className="bg-white rounded-lg shadow-md">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex">
            <button className="border-b-2 border-blue-500 py-4 px-6 text-sm font-medium text-blue-600">
              Branding
            </button>
          </nav>
        </div>

        <div className="p-6">
          {formErrors.general && (
            <div className="mb-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded text-sm">
              {formErrors.general}
            </div>
          )}

          <form onSubmit={handleSubmit}>
            <div className="space-y-6">
              <div>
                <label
                  htmlFor="company_name"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Company Name *
                </label>
                <input
                  type="text"
                  id="company_name"
                  name="company_name"
                  value={formData.company_name}
                  onChange={handleInputChange}
                  className={`w-full max-w-md px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    formErrors.company_name ? 'border-red-500' : 'border-gray-300'
                  }`}
                />
                {formErrors.company_name && (
                  <p className="mt-1 text-sm text-red-600">{formErrors.company_name}</p>
                )}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div>
                  <label
                    htmlFor="primary_color"
                    className="block text-sm font-medium text-gray-700 mb-1"
                  >
                    Primary Color
                  </label>
                  <div className="flex items-center space-x-3">
                    <input
                      type="color"
                      id="primary_color"
                      name="primary_color"
                      value={formData.primary_color}
                      onChange={handleInputChange}
                      className="h-10 w-14 cursor-pointer border border-gray-300 rounded"
                    />
                    <input
                      type="text"
                      name="primary_color"
                      value={formData.primary_color}
                      onChange={handleInputChange}
                      className={`flex-1 px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                        formErrors.primary_color ? 'border-red-500' : 'border-gray-300'
                      }`}
                      placeholder="#3B82F6"
                    />
                  </div>
                  {formErrors.primary_color && (
                    <p className="mt-1 text-sm text-red-600">{formErrors.primary_color}</p>
                  )}
                </div>

                <div>
                  <label
                    htmlFor="secondary_color"
                    className="block text-sm font-medium text-gray-700 mb-1"
                  >
                    Secondary Color
                  </label>
                  <div className="flex items-center space-x-3">
                    <input
                      type="color"
                      id="secondary_color"
                      name="secondary_color"
                      value={formData.secondary_color}
                      onChange={handleInputChange}
                      className="h-10 w-14 cursor-pointer border border-gray-300 rounded"
                    />
                    <input
                      type="text"
                      name="secondary_color"
                      value={formData.secondary_color}
                      onChange={handleInputChange}
                      className={`flex-1 px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                        formErrors.secondary_color ? 'border-red-500' : 'border-gray-300'
                      }`}
                      placeholder="#6B7280"
                    />
                  </div>
                  {formErrors.secondary_color && (
                    <p className="mt-1 text-sm text-red-600">{formErrors.secondary_color}</p>
                  )}
                </div>

                <div>
                  <label
                    htmlFor="accent_color"
                    className="block text-sm font-medium text-gray-700 mb-1"
                  >
                    Accent Color
                  </label>
                  <div className="flex items-center space-x-3">
                    <input
                      type="color"
                      id="accent_color"
                      name="accent_color"
                      value={formData.accent_color}
                      onChange={handleInputChange}
                      className="h-10 w-14 cursor-pointer border border-gray-300 rounded"
                    />
                    <input
                      type="text"
                      name="accent_color"
                      value={formData.accent_color}
                      onChange={handleInputChange}
                      className={`flex-1 px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                        formErrors.accent_color ? 'border-red-500' : 'border-gray-300'
                      }`}
                      placeholder="#10B981"
                    />
                  </div>
                  {formErrors.accent_color && (
                    <p className="mt-1 text-sm text-red-600">{formErrors.accent_color}</p>
                  )}
                </div>
              </div>

              <div>
                <label
                  htmlFor="support_email"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Support Email
                </label>
                <input
                  type="email"
                  id="support_email"
                  name="support_email"
                  value={formData.support_email}
                  onChange={handleInputChange}
                  className={`w-full max-w-md px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    formErrors.support_email ? 'border-red-500' : 'border-gray-300'
                  }`}
                  placeholder="support@example.com"
                />
                {formErrors.support_email && (
                  <p className="mt-1 text-sm text-red-600">{formErrors.support_email}</p>
                )}
              </div>

              <div>
                <label
                  htmlFor="support_phone"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Support Phone
                </label>
                <input
                  type="tel"
                  id="support_phone"
                  name="support_phone"
                  value={formData.support_phone}
                  onChange={handleInputChange}
                  className={`w-full max-w-md px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    formErrors.support_phone ? 'border-red-500' : 'border-gray-300'
                  }`}
                  placeholder="+1 (555) 123-4567"
                />
                {formErrors.support_phone && (
                  <p className="mt-1 text-sm text-red-600">{formErrors.support_phone}</p>
                )}
              </div>
            </div>

            <div className="mt-8 flex justify-end">
              <button
                type="submit"
                disabled={isSaving}
                className="px-6 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSaving ? 'Saving...' : 'Save Changes'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
