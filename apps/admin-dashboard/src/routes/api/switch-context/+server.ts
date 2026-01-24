import { json } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { api, ApiError } from "$lib/api";
import { setAuthCookie } from "$lib/cookies";

interface MembershipInfo {
  id: string;
  organization_id: string;
  organization_name: string;
  organization_slug: string;
  role: string;
  is_default: boolean;
}

interface SwitchContextResponse {
  token: string;
  membership: MembershipInfo;
}

export const POST: RequestHandler = async ({ request, cookies }) => {
  const token = cookies.get("token");

  if (!token) {
    return json({ error: "Not authenticated" }, { status: 401 });
  }

  try {
    const { membership_id } = await request.json();

    if (!membership_id) {
      return json({ error: "membership_id is required" }, { status: 400 });
    }

    // Call backend to switch context
    const response = await api.post<SwitchContextResponse>(
      "/contexts/switch",
      { membership_id },
      token
    );

    const host = request.headers.get("host") || "";

    // Update token cookie with new context (shared across subdomains)
    setAuthCookie(cookies, "token", response.token, host, true);

    // Update membership cookie (shared across subdomains)
    setAuthCookie(cookies, "membership", JSON.stringify(response.membership), host, false);

    return json({ success: true, membership: response.membership });
  } catch (err) {
    if (err instanceof ApiError) {
      return json({ error: err.message }, { status: err.status });
    }
    return json({ error: "Failed to switch context" }, { status: 500 });
  }
};
