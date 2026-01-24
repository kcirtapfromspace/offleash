import { redirect, fail } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";
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

interface LoginResponse {
  token: string;
  user: UserInfo;
  membership?: MembershipInfo;
  memberships?: MembershipInfo[];
}

export const load: PageServerLoad = async ({ cookies }) => {
  const token = cookies.get("token");
  if (token) {
    throw redirect(303, "/dashboard");
  }
  return {};
};

export const actions: Actions = {
  default: async ({ request, cookies }) => {
    const data = await request.formData();
    const email = data.get("email")?.toString();
    const password = data.get("password")?.toString();

    if (!email || !password) {
      return fail(400, { error: "Email and password are required", email });
    }

    try {
      // Use universal login
      const response = await api.post<LoginResponse>("/auth/login/universal", {
        email,
        password,
      });

      // Check if user has admin/owner role in any membership
      const adminMemberships = response.memberships?.filter(
        (m) => m.role === "admin" || m.role === "owner"
      ) ?? [];

      if (adminMemberships.length === 0) {
        return fail(403, {
          error: "You do not have admin access to any organization",
          email,
        });
      }

      const host = request.headers.get("host") || "";

      // Store token (shared across subdomains)
      setAuthCookie(cookies, "token", response.token, host, true);

      // Store user info (shared across subdomains)
      setAuthCookie(cookies, "user", JSON.stringify(response.user), host, false);

      // Store current membership (shared across subdomains)
      if (response.membership) {
        setAuthCookie(cookies, "membership", JSON.stringify(response.membership), host, false);
      }

      // Store admin memberships only (shared across subdomains)
      setAuthCookie(cookies, "memberships", JSON.stringify(adminMemberships), host, false);

      throw redirect(303, "/dashboard");
    } catch (err) {
      if (err instanceof ApiError) {
        if (err.status === 401) {
          return fail(401, { error: "Invalid email or password", email });
        }
        return fail(err.status, {
          error: err.message || "An error occurred. Please try again.",
          email,
        });
      }
      // Re-throw redirects
      throw err;
    }
  },
};
