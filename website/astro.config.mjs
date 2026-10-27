import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';
import cloudflare from '@astrojs/cloudflare';
import preact from '@astrojs/preact';

export default defineConfig({
  output: 'server',
  adapter: cloudflare({imageService:'compile'}),
  devToolbar: { enabled: false },
  vite: {
    build: { assetsInlineLimit: 0 },
    plugins: [tailwindcss()],
    server: { strictPort: true },
    resolve: { dedupe: ['preact'] },
    optimizeDeps: { include: ['preact/devtools', 'preact', 'preact/hooks', 'preact/jsx-runtime'] },
  },
  integrations: [preact(), {
    name: 'kortex-development-preview',
    hooks: {
      'astro:config:setup': ({ command, injectRoute }) => {
        if (command === 'dev') for (const page of ['design','components']) injectRoute({
          pattern: `/__dev/${page}`,
          entrypoint: new URL(`./src/previews/${page}.astro`, import.meta.url),
          prerender: false,
        });
      },
    },
  }],
});
