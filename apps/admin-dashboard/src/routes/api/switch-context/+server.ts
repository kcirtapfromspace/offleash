import { json } from "@sveltejs/kit";
import { dev } from "$app/environment";
import type { RequestHandler } from "./$types";
import { api, ApiError } from "$lib/api";

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

    // Update token cookie with new context
    cookies.set("token", response.token, {
      path: "/",
      httpOnly: true,
      secure: !dev,
      sameSite: "lax",
      maxAge: 60 * 60 * 24 * 7,
    });

    // Update membership cookie
    cookies.set("membership", JSON.stringify(response.membership), {
      path: "/",
      httpOnly: false,
      secure: !dev,
      sameSite: "lax",
      maxAge: 60 * 60 * 24 * 7,
    });

    return json({ success: true, membership: response.membership });
  } catch (err) {
    if (err instanceof ApiError) {
      return json({ error: err.message }, { status: err.status });
    }
    return json({ error: "Failed to switch context" }, { status: 500 });
  }
};
