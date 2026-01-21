import { redirect, fail } from "@sveltejs/kit";
import { dev } from "$app/environment";
import type { Actions, PageServerLoad } from "./$types";
import { api, ApiError } from "$lib/api";

interface LoginResponse {
  token: string;
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
      const response = await api.post<LoginResponse>("/auth/login", {
        org_slug: "demo",
        email,
        password,
      });

      cookies.set("token", response.token, {
        path: "/",
        httpOnly: true,
        secure: !dev,
        sameSite: "lax",
        maxAge: 60 * 60 * 24 * 7, // 7 days
      });

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
