// astro.config.mjs
import { defineConfig } from 'astro/config';
import react from '@astrojs/react'; // ✅ import the integration
import vercel from '@astrojs/vercel/serverless'; // 👈 enable SSR


export default defineConfig({
  output: 'server', // required for server routes like /api/*
  adapter: vercel(),
  integrations: [react()], // ✅ enable it here
});
