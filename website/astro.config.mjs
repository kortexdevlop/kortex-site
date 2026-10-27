import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  output: 'static',
  devToolbar: { enabled: false },
  vite: { plugins: [tailwindcss()], server: { strictPort: true } },
  integrations: [{
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
