import { test as setup, expect } from '@playwright/test';
import { TEST_USERS, API_ENDPOINTS } from './utils/constants';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

// ES module dirname equivalent
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const authDir = path.join(__dirname, '.auth');

// Ensure auth directory exists
if (!fs.existsSync(authDir)) {
  fs.mkdirSync(authDir, { recursive: true });
}

/**
 * Global setup to create authentication state files for each user role.
 * These are used by browser tests to skip login for each test.
 */

setup('authenticate customer', async ({ request }) => {
  // Login as customer
  const loginResponse = await request.post(API_ENDPOINTS.LOGIN, {
    data: {
      email: TEST_USERS.customer.email,
      password: TEST_USERS.customer.password,
    },
  });

  expect(loginResponse.ok()).toBeTruthy();
  const loginData = await loginResponse.json();

  // Switch context if needed
  let token = loginData.token;
  if (loginData.memberships && loginData.memberships.length > 0) {
    const defaultMembership = loginData.memberships.find((m: any) => m.is_default) || loginData.memberships[0];
    const switchResponse = await request.post(API_ENDPOINTS.SWITCH_CONTEXT, {
      headers: { Authorization: `Bearer ${token}` },
      data: { membership_id: defaultMembership.id },
    });
    expect(switchResponse.ok()).toBeTruthy();
    const switchData = await switchResponse.json();
    token = switchData.token;
  }

  // Save storage state with token
  const storageState = {
    cookies: [],
    origins: [
      {
        origin: process.env.CUSTOMER_URL || 'http://localhost:5173',
        localStorage: [
          { name: 'auth_token', value: token },
          { name: 'user', value: JSON.stringify(loginData.user) },
        ],
      },
    ],
  };

  fs.writeFileSync(path.join(authDir, 'customer.json'), JSON.stringify(storageState, null, 2));
});

setup('authenticate walker', async ({ request }) => {
  const loginResponse = await request.post(API_ENDPOINTS.LOGIN, {
    data: {
      email: TEST_USERS.walker.email,
      password: TEST_USERS.walker.password,
    },
  });

  expect(loginResponse.ok()).toBeTruthy();
  const loginData = await loginResponse.json();

  let token = loginData.token;
  if (loginData.memberships && loginData.memberships.length > 0) {
    const walkerMembership = loginData.memberships.find((m: any) => m.role === 'walker') || loginData.memberships[0];
    const switchResponse = await request.post(API_ENDPOINTS.SWITCH_CONTEXT, {
      headers: { Authorization: `Bearer ${token}` },
      data: { membership_id: walkerMembership.id },
    });
    expect(switchResponse.ok()).toBeTruthy();
    const switchData = await switchResponse.json();
    token = switchData.token;
  }

  const storageState = {
    cookies: [],
    origins: [
      {
        origin: process.env.CUSTOMER_URL || 'http://localhost:5173',
        localStorage: [
          { name: 'auth_token', value: token },
          { name: 'user', value: JSON.stringify(loginData.user) },
        ],
      },
    ],
  };

  fs.writeFileSync(path.join(authDir, 'walker.json'), JSON.stringify(storageState, null, 2));
});

setup('authenticate admin', async ({ request }) => {
  const loginResponse = await request.post(API_ENDPOINTS.LOGIN, {
    data: {
      email: TEST_USERS.admin.email,
      password: TEST_USERS.admin.password,
    },
  });

  expect(loginResponse.ok()).toBeTruthy();
  const loginData = await loginResponse.json();

  let token = loginData.token;
  let currentMembership = loginData.membership;

  if (loginData.memberships && loginData.memberships.length > 0) {
    const adminMembership =
      loginData.memberships.find((m: any) => m.role === 'admin' || m.role === 'owner') || loginData.memberships[0];
    const switchResponse = await request.post(API_ENDPOINTS.SWITCH_CONTEXT, {
      headers: { Authorization: `Bearer ${token}` },
      data: { membership_id: adminMembership.id },
    });
    expect(switchResponse.ok()).toBeTruthy();
    const switchData = await switchResponse.json();
    token = switchData.token;
    currentMembership = switchData.membership || adminMembership;
  }

  const adminUrl = process.env.ADMIN_URL || 'http://localhost:5174';
  const urlObj = new URL(adminUrl);
  const domain = urlObj.hostname;

  // SvelteKit uses cookies for server-side auth, so we need to set cookies too
  const storageState = {
    cookies: [
      {
        name: 'token',
        value: token,
        domain: domain,
        path: '/',
        expires: Date.now() / 1000 + 86400, // 24 hours from now
        httpOnly: true,
        secure: false,
        sameSite: 'Lax' as const,
      },
      {
        name: 'user',
        value: JSON.stringify(loginData.user),
        domain: domain,
        path: '/',
        expires: Date.now() / 1000 + 86400,
        httpOnly: false,
        secure: false,
        sameSite: 'Lax' as const,
      },
      {
        name: 'membership',
        value: JSON.stringify(currentMembership),
        domain: domain,
        path: '/',
        expires: Date.now() / 1000 + 86400,
        httpOnly: false,
        secure: false,
        sameSite: 'Lax' as const,
      },
      {
        name: 'memberships',
        value: JSON.stringify(loginData.memberships?.filter((m: any) =>
          m.role === 'admin' || m.role === 'owner' || m.role === 'walker'
        ) || []),
        domain: domain,
        path: '/',
        expires: Date.now() / 1000 + 86400,
        httpOnly: false,
        secure: false,
        sameSite: 'Lax' as const,
      },
    ],
    origins: [
      {
        origin: adminUrl,
        localStorage: [
          { name: 'auth_token', value: token },
          { name: 'user', value: JSON.stringify(loginData.user) },
        ],
      },
    ],
  };

  fs.writeFileSync(path.join(authDir, 'admin.json'), JSON.stringify(storageState, null, 2));
});

setup('authenticate platform admin', async ({ request }) => {
  const loginResponse = await request.post(API_ENDPOINTS.PLATFORM_LOGIN, {
    data: {
      email: TEST_USERS.platformAdmin.email,
      password: TEST_USERS.platformAdmin.password,
    },
  });

  expect(loginResponse.ok()).toBeTruthy();
  const loginData = await loginResponse.json();

  const storageState = {
    cookies: [],
    origins: [
      {
        origin: process.env.PLATFORM_URL || 'http://localhost:5175',
        localStorage: [
          { name: 'auth_token', value: loginData.token },
          { name: 'user', value: JSON.stringify(loginData.user) },
        ],
      },
    ],
  };

  fs.writeFileSync(path.join(authDir, 'platform.json'), JSON.stringify(storageState, null, 2));
});
