import { NavLink } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';

const navItems = [
  { path: '/dashboard', label: 'Dashboard' },
  { path: '/walkers', label: 'Walkers' },
  { path: '/services', label: 'Services' },
  { path: '/bookings', label: 'Bookings' },
  { path: '/customers', label: 'Customers' },
  { path: '/settings', label: 'Settings' },
];

export function Navigation() {
  const { logout } = useAuth();

  return (
    <nav className="bg-gray-800 text-white p-4">
      <div className="flex items-center justify-between">
        <div className="flex space-x-4">
          {navItems.map((item) => (
            <NavLink
              key={item.path}
              to={item.path}
              className={({ isActive }) =>
                `px-3 py-2 rounded ${isActive ? 'bg-gray-900' : 'hover:bg-gray-700'}`
              }
            >
              {item.label}
            </NavLink>
          ))}
        </div>
        <button
          onClick={logout}
          className="px-3 py-2 rounded bg-red-600 hover:bg-red-700"
        >
          Logout
        </button>
      </div>
    </nav>
  );
}
