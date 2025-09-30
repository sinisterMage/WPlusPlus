export const prerender = true;
const BASE_URL = "https://wplusplus.org";

const pages = [
  "", // home
  "syntax",
  "faq",
  "contribute",
  "dev",
  "mascot",
  "contact",
];

export async function GET() {
  const lastmod = new Date().toISOString();

  const urls = pages.map(
    (slug) => `
  <url>
    <loc>${BASE_URL}/${slug}</loc>
    <lastmod>${lastmod}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>${slug === "" ? "1.0" : "0.7"}</priority>
  </url>`
  );

  return new Response(
    `<?xml version="1.0" encoding="UTF-8"?>
  <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  ${urls.join("\n")}
  </urlset>`,
    {
      headers: {
        "Content-Type": "application/xml",
      },
    }
  );
}
