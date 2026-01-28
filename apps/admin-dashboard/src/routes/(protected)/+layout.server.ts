import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";
import { api, ApiError } from "$lib/api";
import { setAuthCookie, deleteAuthCookie } from "$lib/cookies";

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

  // Check if token has been upgraded to include org_id
  // If not, we need to fetch session and potentially switch context
  const tokenHasOrgId = cookies.get("token_has_org_id");
  if (!tokenHasOrgId) {
    needsSessionFetch = true;
  }

  // If we're missing user or memberships info, fetch from API session endpoint
  // This handles cross-subdomain SSO where we have a token but no user cookies
  let currentToken = token;
  if (needsSessionFetch) {
    try {
      const session = await api.get<SessionResponse>("/auth/session", currentToken);
      user = session.user;
      membership = session.membership || null;
      memberships = session.memberships || [];

      // If token doesn't have org_id but user has a membership, switch context to get a new token
      // This handles tokens created before org_id was required
      const host = request.headers.get("host") || "";
      if (!session.org_id && membership) {
        try {
          console.log("Token missing org_id, switching context to membership:", membership.id);
          const switchResponse = await api.post<{ token: string; membership: MembershipInfo }>(
            "/contexts/switch",
            { membership_id: membership.id },
            currentToken
          );
          currentToken = switchResponse.token;

          // Update the token cookie with the new token that has org_id
          setAuthCookie(cookies, "token", currentToken, host, true);
          // Mark that token now has org_id
          setAuthCookie(cookies, "token_has_org_id", "true", host, false);
          console.log("Context switch successful in layout");
        } catch (switchErr) {
          console.error("Failed to switch context in layout:", switchErr);
          // If context switch fails, clear cookies and redirect to login
          // This forces a fresh login which should properly set up the token
          deleteAuthCookie(cookies, "token", host);
          deleteAuthCookie(cookies, "user", host);
          deleteAuthCookie(cookies, "membership", host);
          deleteAuthCookie(cookies, "memberships", host);
          deleteAuthCookie(cookies, "token_has_org_id", host);
          throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
        }
      } else if (session.org_id) {
        // Token already has org_id, mark it
        setAuthCookie(cookies, "token_has_org_id", "true", host, false);
      }

      // Update cookies with session data for faster subsequent loads
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
        // Token is invalid or expired - delete it and redirect to login
        const host = request.headers.get("host") || "";
        deleteAuthCookie(cookies, "token", host);
        deleteAuthCookie(cookies, "user", host);
        deleteAuthCookie(cookies, "membership", host);
        deleteAuthCookie(cookies, "memberships", host);
        deleteAuthCookie(cookies, "token_has_org_id", host);
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
    hasToken: !!currentToken,
    hasUser: !!user,
    hasMembership: !!membership,
    membershipRole: membership?.role,
    membershipOrgId: membership?.organization_id,
    membershipsCount: memberships.length
  });

  return {
    token: currentToken,
    user,
    membership,
    memberships,
  };
};
