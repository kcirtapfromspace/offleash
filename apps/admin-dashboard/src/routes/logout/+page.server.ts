import { redirect } from "@sveltejs/kit";
import type { Actions } from "./$types";
import { deleteAuthCookie } from "$lib/cookies";

export const actions: Actions = {
  default: async ({ cookies, request }) => {
    const host = request.headers.get("host") || "";

    // Delete cookies with proper domain for cross-subdomain clearing
    deleteAuthCookie(cookies, "token", host);
    deleteAuthCookie(cookies, "user", host);
    deleteAuthCookie(cookies, "membership", host);
    deleteAuthCookie(cookies, "memberships", host);

    throw redirect(303, "/login");
  },
};
