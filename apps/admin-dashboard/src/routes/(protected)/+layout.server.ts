import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";
import { api } from "$lib/api";

interface UserInfo {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  role: string;
}

export const load: LayoutServerLoad = async ({ cookies, url }) => {
  const token = cookies.get("token");

  if (!token) {
    throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
  }

  // Parse user info from cookie
  let user: UserInfo | null = null;
  const userCookie = cookies.get("user");
  if (userCookie) {
    try {
      user = JSON.parse(userCookie);
    } catch {
      // Invalid user cookie, will fetch from API
    }
  }

  // If no user cookie, fetch from API
  if (!user) {
    try {
      user = await api.get<UserInfo>("/users/me", token);
    } catch {
      // Failed to fetch user info, redirect to login
      throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
    }
  }

  return {
    token,
    user,
  };
};
