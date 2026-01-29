import { environment } from '../config/environments.js';

const headers: Record<string, string> = {
	'content-type': 'application/json',
};

if (process.env.TEST_API_TOKEN) {
	headers.authorization = `Bearer ${process.env.TEST_API_TOKEN}`;
}

async function post(path: string, body: Record<string, unknown> = {}) {
	const url = `${environment.apiBaseUrl}${path}`;
	const response = await fetch(url, {
		method: 'POST',
		headers,
		body: JSON.stringify(body),
	});

	return { ok: response.ok, status: response.status };
}

export async function seedTestData() {
	try {
		await post('/test/seed');
	} catch {
		// Best-effort; test env may not expose seed endpoint
	}
}

export async function cleanupTestData() {
	try {
		await post('/test/cleanup');
	} catch {
		// Best-effort; avoid failing test suite on cleanup
	}
}

export async function expireSession() {
	try {
		await post('/test/expire-session');
	} catch {
		// Best-effort
	}
}
