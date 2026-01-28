import { defineConfig, devices } from '@playwright/test';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';

// ES module dirname equivalent
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load test environment variables (prefer local config)
dotenv.config({ path: path.resolve(__dirname, '.env.test.local') });
dotenv.config({ path: path.resolve(__dirname, '.env.test') });

const isCI = !!process.env.CI;
const isStaging = !!process.env.STAGING;

// Base URLs for different environments
const LOCAL_API_URL = 'http://localhost:8080';
const LOCAL_CUSTOMER_URL = 'http://localhost:5173';
const LOCAL_ADMIN_URL = 'http://localhost:5174';
const LOCAL_PLATFORM_URL = 'http://localhost:5175';

const STAGING_API_URL = process.env.STAGING_API_URL || 'https://api.offleash.world';
const STAGING_CUSTOMER_URL = process.env.STAGING_URL || 'https://offleash.world';
const STAGING_ADMIN_URL = process.env.STAGING_ADMIN_URL || 'https://paperwork.offleash.world';
const STAGING_PLATFORM_URL = process.env.STAGING_PLATFORM_URL || 'https://platform.offleash.world';

export default defineConfig({
  testDir: './tests',

  // Run tests in parallel
  fullyParallel: true,

  // Fail fast in CI
  forbidOnly: isCI,

  // Retries
  retries: isCI ? 2 : 0,

  // Workers - use more in CI with sharding
  workers: isCI ? 4 : undefined,

  // Reporter
  reporter: isCI
    ? [['github'], ['html', { open: 'never' }], ['json', { outputFile: 'test-results.json' }]]
    : [['list'], ['html', { open: 'on-failure' }]],

  // Global timeout
  timeout: 30000,

  // Expect timeout
  expect: {
    timeout: 10000,
  },

  // Shared settings for all projects
  use: {
    // Base URL is set per project
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',

    // Default headers for API requests
    extraHTTPHeaders: {
      'Content-Type': 'application/json',
    },
  },

  // Test projects
  projects: [
    // API Tests - run first, no browser needed
    {
      name: 'api',
      testDir: './tests/api',
      use: {
        baseURL: isStaging ? STAGING_API_URL : LOCAL_API_URL,
      },
    },

    // Demo Tests (API) - for route optimization demos
    {
      name: 'demos',
      testDir: './tests/demos/api',
      use: {
        baseURL: isStaging ? STAGING_API_URL : LOCAL_API_URL,
        video: 'on',
        screenshot: 'on',
      },
    },

    // Demo Tests (UI) - for browser-based demos with video
    {
      name: 'demos-ui',
      testDir: './tests/demos/ui',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: isStaging ? STAGING_ADMIN_URL : LOCAL_ADMIN_URL,
        storageState: 'tests/.auth/admin.json',
        video: 'on',
        screenshot: 'on',
      },
      dependencies: ['setup'],
    },

    // Setup project - runs before browser tests
    {
      name: 'setup',
      testMatch: /global\.setup\.ts/,
      use: {
        baseURL: isStaging ? STAGING_API_URL : LOCAL_API_URL,
      },
    },

    // Customer Web - Chromium
    {
      name: 'customer-web',
      testDir: './tests/e2e/customer-web',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: isStaging ? STAGING_CUSTOMER_URL : LOCAL_CUSTOMER_URL,
        storageState: 'tests/.auth/customer.json',
      },
      dependencies: ['setup'],
    },

    // Customer Web - Firefox
    {
      name: 'customer-web-firefox',
      testDir: './tests/e2e/customer-web',
      use: {
        ...devices['Desktop Firefox'],
        baseURL: isStaging ? STAGING_CUSTOMER_URL : LOCAL_CUSTOMER_URL,
        storageState: 'tests/.auth/customer.json',
      },
      dependencies: ['setup'],
    },

    // Customer Web - WebKit (Safari)
    {
      name: 'customer-web-webkit',
      testDir: './tests/e2e/customer-web',
      use: {
        ...devices['Desktop Safari'],
        baseURL: isStaging ? STAGING_CUSTOMER_URL : LOCAL_CUSTOMER_URL,
        storageState: 'tests/.auth/customer.json',
      },
      dependencies: ['setup'],
    },

    // Admin Dashboard - Chromium
    {
      name: 'admin-dashboard',
      testDir: './tests/e2e/admin-dashboard',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: isStaging ? STAGING_ADMIN_URL : LOCAL_ADMIN_URL,
        storageState: 'tests/.auth/admin.json',
      },
      dependencies: ['setup'],
    },

    // Platform Admin - Chromium
    {
      name: 'platform-admin',
      testDir: './tests/e2e/platform-admin',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: isStaging ? STAGING_PLATFORM_URL : LOCAL_PLATFORM_URL,
        storageState: 'tests/.auth/platform.json',
      },
      dependencies: ['setup'],
    },

    // Staging smoke tests
    {
      name: 'staging',
      testDir: './tests/staging',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: STAGING_CUSTOMER_URL,
      },
    },
  ],

  // Web server configuration for local testing
  // Note: API server must be running on port 8080
  // Start with: npm run dev:api (or cargo run from crates/api)
  webServer: isCI || isStaging ? undefined : [
    // Rust API server - uses cargo run
    {
      command: 'cd crates/api && cargo run',
      port: 8080,
      reuseExistingServer: true,
      timeout: 180000, // Rust compilation can take time
      stdout: 'pipe',
      stderr: 'pipe',
    },
    {
      command: 'cd apps/customer-web && npm run dev -- --port 5173',
      port: 5173,
      reuseExistingServer: true,
      timeout: 120000,
    },
    {
      command: 'cd apps/admin-dashboard && npm run dev -- --port 5174',
      port: 5174,
      reuseExistingServer: true,
      timeout: 120000,
    },
    {
      command: 'cd apps/platform-admin && npm run dev -- --port 5175',
      port: 5175,
      reuseExistingServer: true,
      timeout: 120000,
    },
  ],

  // Output directory for test artifacts
  outputDir: 'test-results/',
});
