import { browser } from '$app/environment';
import { PUBLIC_API_URL } from '$env/static/public';

const API_BASE = browser ? '' : PUBLIC_API_URL;

export class ApiError extends Error {
	status: number;
	statusText: string;

	constructor(status: number, statusText: string, message?: string) {
		super(message ?? `${status} ${statusText}`);
		this.name = 'ApiError';
		this.status = status;
		this.statusText = statusText;
	}
}

interface RequestOptions extends RequestInit {
	token?: string;
}

async function request<T>(url: string, options: RequestOptions = {}): Promise<T> {
	const { token, ...fetchOptions } = options;

	const headers: HeadersInit = {
		'Content-Type': 'application/json',
		...fetchOptions.headers
	};

	if (token) {
		(headers as Record<string, string>)['Authorization'] = `Bearer ${token}`;
	}

	const fullUrl = url.startsWith('http') ? url : `${API_BASE}${url}`;
	const response = await fetch(fullUrl, {
		...fetchOptions,
		headers
	});

	if (response.status === 401) {
		if (browser) {
			window.location.href = '/login';
		}
		throw new ApiError(401, 'Unauthorized', 'Session expired');
	}

	if (!response.ok) {
		const message = await response.text().catch(() => undefined);
		throw new ApiError(response.status, response.statusText, message);
	}

	if (response.status === 204) {
		return undefined as T;
	}

	return response.json() as Promise<T>;
}

export async function get<T>(url: string, token?: string): Promise<T> {
	return request<T>(url, { method: 'GET', token });
}

export async function post<T>(url: string, body?: unknown, token?: string): Promise<T> {
	return request<T>(url, {
		method: 'POST',
		body: body !== undefined ? JSON.stringify(body) : undefined,
		token
	});
}

export async function put<T>(url: string, body?: unknown, token?: string): Promise<T> {
	return request<T>(url, {
		method: 'PUT',
		body: body !== undefined ? JSON.stringify(body) : undefined,
		token
	});
}

export async function patch<T>(url: string, body?: unknown, token?: string): Promise<T> {
	return request<T>(url, {
		method: 'PATCH',
		body: body !== undefined ? JSON.stringify(body) : undefined,
		token
	});
}

export async function del<T>(url: string, token?: string): Promise<T> {
	return request<T>(url, { method: 'DELETE', token });
}

export const api = {
	get,
	post,
	put,
	patch,
	delete: del
};
