import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";
import { api, ApiError } from "$lib/api";
import { setAuthCookie } from "$lib/cookies";

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

interface SessionResponse {
  user: UserInfo;
  membership?: MembershipInfo;
  memberships?: MembershipInfo[];
  org_id?: string;
}

export const load: LayoutServerLoad = async ({ cookies, url, request }) => {
  const token = cookies.get("token");

  if (!token) {
    throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
  }

  // Try to get session info from cookies first (faster)
  let user: UserInfo | null = null;
  let membership: MembershipInfo | null = null;
  let memberships: MembershipInfo[] = [];
  let needsSessionFetch = false;

  const userCookie = cookies.get("user");
  if (userCookie) {
    try {
      user = JSON.parse(userCookie);
    } catch {
      needsSessionFetch = true;
    }
  } else {
    needsSessionFetch = true;
  }

  const membershipCookie = cookies.get("membership");
  if (membershipCookie) {
    try {
      membership = JSON.parse(membershipCookie);
    } catch {
      // Invalid membership cookie - will fetch from API
    }
  }

  const membershipsCookie = cookies.get("memberships");
  if (membershipsCookie) {
    try {
      memberships = JSON.parse(membershipsCookie);
    } catch {
      needsSessionFetch = true;
    }
  } else {
    needsSessionFetch = true;
  }

  // If we're missing user or memberships info, fetch from API session endpoint
  // This handles cross-subdomain SSO where we have a token but no user cookies
  if (needsSessionFetch) {
    try {
      const session = await api.get<SessionResponse>("/auth/session", token);
      user = session.user;
      membership = session.membership || null;
      memberships = session.memberships || [];

      // Update cookies with session data for faster subsequent loads
      const host = request.headers.get("host") || "";
      setAuthCookie(cookies, "user", JSON.stringify(user), host, false);
      if (membership) {
        setAuthCookie(cookies, "membership", JSON.stringify(membership), host, false);
      }
      if (memberships.length > 0) {
        // For admin dashboard, store admin/owner/walker memberships (staff roles)
        const staffMemberships = memberships.filter(
          (m) => m.role === "admin" || m.role === "owner" || m.role === "walker"
        );
        setAuthCookie(cookies, "memberships", JSON.stringify(staffMemberships), host, false);
        memberships = staffMemberships;
      }
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        // Token is invalid or expired - redirect to login
        throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
      }
      throw err;
    }
  }

  // Verify user has staff access (admin, owner, or walker) to at least one organization
  const hasStaffAccess = memberships.some(
    (m) => m.role === "admin" || m.role === "owner" || m.role === "walker"
  );

  if (!hasStaffAccess && user) {
    // User authenticated but no staff access - redirect to customer web
    throw redirect(303, "https://offleash.world/services");
  }

  // Debug logging to help diagnose API issues
  console.log('Layout data:', {
    hasToken: !!token,
    hasUser: !!user,
    hasMembership: !!membership,
    membershipRole: membership?.role,
    membershipOrgId: membership?.organization_id,
    membershipsCount: memberships.length
  });

  return {
    token,
    user,
    membership,
    memberships,
  };
};
