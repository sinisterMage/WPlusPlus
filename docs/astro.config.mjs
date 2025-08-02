// astro.config.mjs
import { defineConfig } from 'astro/config';
import react from '@astrojs/react'; // ✅ import the integration

export default defineConfig({
  integrations: [react()], // ✅ enable it here
});
