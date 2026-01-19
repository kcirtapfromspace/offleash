import { Outlet, useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { Navigation } from './Navigation';

export function DashboardLayout() {
  const { logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="min-h-screen bg-gray-100 flex">
      {/* Sidebar */}
      <aside className="w-64 bg-gray-800 text-white flex flex-col">
        {/* Company name / Logo area */}
        <div className="h-16 flex items-center justify-center border-b border-gray-700">
          <h1 className="text-xl font-bold">OFFLEASH</h1>
        </div>

        {/* Navigation */}
        <Navigation />

        {/* Logout button at bottom */}
        <div className="p-4 border-t border-gray-700">
          <button
            onClick={handleLogout}
            className="w-full flex items-center px-4 py-3 rounded-lg text-gray-300 hover:bg-red-600 hover:text-white transition-colors"
          >
            <svg
              className="w-5 h-5 mr-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
              />
            </svg>
            Logout
          </button>
        </div>
      </aside>

      {/* Main content area */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <header className="h-16 bg-white shadow-sm flex items-center justify-between px-6">
          <h2 className="text-lg font-semibold text-gray-800">Admin Dashboard</h2>
          <div className="flex items-center space-x-4">
            <button
              onClick={handleLogout}
              className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 transition-colors"
            >
              Logout
            </button>
          </div>
        </header>

        {/* Page content */}
        <main className="flex-1 overflow-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
