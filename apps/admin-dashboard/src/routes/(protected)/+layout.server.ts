import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = async ({ cookies, url }) => {
  const token = cookies.get("token");

  if (!token) {
    throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
  }

  // TODO: Optionally verify token and fetch user info here
  return {
    token,
  };
};
