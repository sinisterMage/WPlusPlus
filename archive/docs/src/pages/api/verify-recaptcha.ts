export const prerender = false;

import type { APIRoute } from "astro";

export const POST: APIRoute = async ({ request }) => {
  let token: string | undefined;

  try {
    const contentType = request.headers.get("content-type") || "";

    if (!contentType.includes("application/json")) {
      return new Response(
        JSON.stringify({ success: false, error: "Invalid content-type" }),
        { status: 400 }
      );
    }

    const rawBody = await request.text();

    if (!rawBody) {
      return new Response(
        JSON.stringify({ success: false, error: "Empty request body" }),
        { status: 400 }
      );
    }


    const parsed = JSON.parse(rawBody);
    token = parsed.token;

    if (!token) {
      return new Response(
        JSON.stringify({ success: false, error: "Missing reCAPTCHA token" }),
        { status: 400 }
      );
    }
  } catch (err) {
    console.error("❌ Failed to parse body:", err);
    return new Response(
      JSON.stringify({ success: false, error: "Invalid JSON body" }),
      { status: 400 }
    );
  }

  const secret = import.meta.env.RECAPTCHA_SECRET;

  if (!secret) {
    return new Response(
      JSON.stringify({ success: false, error: "Missing reCAPTCHA secret" }),
      { status: 500 }
    );
  }

  try {
    const verifyRes = await fetch("https://www.google.com/recaptcha/api/siteverify", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        secret,
        response: token,
      }),
    });

    const data = await verifyRes.json();
    

    if (!data.success) {
      return new Response(
        JSON.stringify({ success: false, error: "reCAPTCHA validation failed" }),
        { status: 403 }
      );
    }

    return new Response(JSON.stringify({ success: true }), { status: 200 });
  } catch (err) {
    console.error("⚠️ Error verifying reCAPTCHA:", err);
    return new Response(
      JSON.stringify({ success: false, error: "reCAPTCHA verification error" }),
      { status: 500 }
    );
  }
};
