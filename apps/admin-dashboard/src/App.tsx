import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './contexts/AuthContext';
import { ProtectedRoute } from './components/ProtectedRoute';
import { Layout } from './components/Layout';
import {
  LoginPage,
  DashboardPage,
  WalkersPage,
  ServicesPage,
  BookingsPage,
  CustomersPage,
  SettingsPage,
} from './pages';

function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route
            element={
              <ProtectedRoute>
                <Layout />
              </ProtectedRoute>
            }
          >
            <Route path="/dashboard" element={<DashboardPage />} />
            <Route path="/walkers" element={<WalkersPage />} />
            <Route path="/services" element={<ServicesPage />} />
            <Route path="/bookings" element={<BookingsPage />} />
            <Route path="/customers" element={<CustomersPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Route>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}

export default App;
