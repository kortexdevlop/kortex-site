import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';
import cloudflare from '@astrojs/cloudflare';
import preact from '@astrojs/preact';

export default defineConfig({
  output: 'server',
  adapter: cloudflare({imageService:'compile'}),
  devToolbar: { enabled: false },
  vite: {
    plugins: [tailwindcss()],
    server: { strictPort: true },
    resolve: { dedupe: ['preact'] },
    optimizeDeps: { include: ['preact/devtools', 'preact', 'preact/hooks', 'preact/jsx-runtime'] },
  },
  integrations: [preact(), {
    name: 'kortex-development-preview',
    hooks: {
      'astro:config:setup': ({ command, injectRoute }) => {
        if (command === 'dev') injectRoute({
          pattern: '/__dev/design',
          entrypoint: new URL('./src/previews/design.astro', import.meta.url),
          prerender: false,
        });
      },
    },
  }],
});
