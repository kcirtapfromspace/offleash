import { APIRequestContext, expect, APIResponse } from '@playwright/test';
import { authHeader } from './auth';

/**
 * API response wrapper with helper methods
 */
export class APIResponseWrapper<T> {
  constructor(
    public readonly response: APIResponse,
    public readonly data: T | null
  ) {}

  get ok(): boolean {
    return this.response.ok();
  }

  get status(): number {
    return this.response.status();
  }

  expectSuccess(): T {
    expect(this.response.ok(), `Expected success but got ${this.status}`).toBeTruthy();
    return this.data as T;
  }

  expectStatus(status: number): this {
    expect(this.response.status()).toBe(status);
    return this;
  }

  expectError(status: number): this {
    expect(this.response.status()).toBe(status);
    return this;
  }
}

/**
 * Wrapper for making authenticated API requests
 */
export class APIClient {
  constructor(
    private request: APIRequestContext,
    private token?: string
  ) {}

  /**
   * Set the auth token
   */
  setToken(token: string): void {
    this.token = token;
  }

  /**
   * Get default headers
   */
  private getHeaders(customHeaders?: Record<string, string>): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...customHeaders,
    };

    if (this.token) {
      headers.Authorization = `Bearer ${this.token}`;
    }

    return headers;
  }

  /**
   * Make a GET request
   */
  async get<T>(url: string, options?: { headers?: Record<string, string> }): Promise<APIResponseWrapper<T>> {
    const response = await this.request.get(url, {
      headers: this.getHeaders(options?.headers),
    });

    let data: T | null = null;
    try {
      if (response.status() !== 204) {
        data = await response.json();
      }
    } catch {
      // Response may not be JSON
    }

    return new APIResponseWrapper(response, data);
  }

  /**
   * Make a POST request
   */
  async post<T>(
    url: string,
    body?: unknown,
    options?: { headers?: Record<string, string> }
  ): Promise<APIResponseWrapper<T>> {
    const response = await this.request.post(url, {
      headers: this.getHeaders(options?.headers),
      data: body,
    });

    let data: T | null = null;
    try {
      if (response.status() !== 204) {
        data = await response.json();
      }
    } catch {
      // Response may not be JSON
    }

    return new APIResponseWrapper(response, data);
  }

  /**
   * Make a PUT request
   */
  async put<T>(
    url: string,
    body?: unknown,
    options?: { headers?: Record<string, string> }
  ): Promise<APIResponseWrapper<T>> {
    const response = await this.request.put(url, {
      headers: this.getHeaders(options?.headers),
      data: body,
    });

    let data: T | null = null;
    try {
      if (response.status() !== 204) {
        data = await response.json();
      }
    } catch {
      // Response may not be JSON
    }

    return new APIResponseWrapper(response, data);
  }

  /**
   * Make a PATCH request
   */
  async patch<T>(
    url: string,
    body?: unknown,
    options?: { headers?: Record<string, string> }
  ): Promise<APIResponseWrapper<T>> {
    const response = await this.request.patch(url, {
      headers: this.getHeaders(options?.headers),
      data: body,
    });

    let data: T | null = null;
    try {
      if (response.status() !== 204) {
        data = await response.json();
      }
    } catch {
      // Response may not be JSON
    }

    return new APIResponseWrapper(response, data);
  }

  /**
   * Make a DELETE request
   */
  async delete<T>(url: string, options?: { headers?: Record<string, string> }): Promise<APIResponseWrapper<T>> {
    const response = await this.request.delete(url, {
      headers: this.getHeaders(options?.headers),
    });

    let data: T | null = null;
    try {
      if (response.status() !== 204) {
        data = await response.json();
      }
    } catch {
      // Response may not be JSON
    }

    return new APIResponseWrapper(response, data);
  }
}

/**
 * Create an API client with the given token
 */
export function createAPIClient(request: APIRequestContext, token?: string): APIClient {
  return new APIClient(request, token);
}

/**
 * Helper to make unauthenticated request
 */
export async function makeRequest<T>(
  request: APIRequestContext,
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE',
  url: string,
  options?: {
    body?: unknown;
    token?: string;
    headers?: Record<string, string>;
  }
): Promise<APIResponseWrapper<T>> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...options?.headers,
  };

  if (options?.token) {
    headers.Authorization = `Bearer ${options.token}`;
  }

  let response: APIResponse;

  switch (method) {
    case 'GET':
      response = await request.get(url, { headers });
      break;
    case 'POST':
      response = await request.post(url, { headers, data: options?.body });
      break;
    case 'PUT':
      response = await request.put(url, { headers, data: options?.body });
      break;
    case 'PATCH':
      response = await request.patch(url, { headers, data: options?.body });
      break;
    case 'DELETE':
      response = await request.delete(url, { headers });
      break;
  }

  let data: T | null = null;
  try {
    if (response.status() !== 204) {
      data = await response.json();
    }
  } catch {
    // Response may not be JSON
  }

  return new APIResponseWrapper(response, data);
}
