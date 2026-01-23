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

interface MembershipInfo {
  id: string;
  organization_id: string;
  organization_name: string;
  organization_slug: string;
  role: string;
  is_default: boolean;
}

export const load: LayoutServerLoad = async ({ cookies, url }) => {
  const token = cookies.get("token");

  if (!token) {
    throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
  }

  // Parse user info from cookie
  let user: UserInfo | null = null;
  let membership: MembershipInfo | null = null;
  let memberships: MembershipInfo[] = [];

  const userCookie = cookies.get("user");
  if (userCookie) {
    try {
      user = JSON.parse(userCookie);
    } catch {
      // Invalid user cookie
    }
  }

  const membershipCookie = cookies.get("membership");
  if (membershipCookie) {
    try {
      membership = JSON.parse(membershipCookie);
    } catch {
      // Invalid membership cookie
    }
  }

  const membershipsCookie = cookies.get("memberships");
  if (membershipsCookie) {
    try {
      memberships = JSON.parse(membershipsCookie);
    } catch {
      // Invalid memberships cookie
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
    membership,
    memberships,
  };
};
