import { writable, derived } from 'svelte/store';

export interface User {
	id: string;
	email: string;
	firstName?: string;
	lastName?: string;
	role: string;
}

export const user = writable<User | null>(null);
export const isAuthenticated = derived(user, ($user) => !!$user);
