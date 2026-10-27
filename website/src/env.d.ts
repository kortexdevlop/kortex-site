declare namespace Cloudflare {
  interface Env { SITEMAP_WORKER?: import('./lib/api-client').ServiceBinding; API_WORKER?: import('./lib/api-client').ServiceBinding; WEB_ORIGIN?: string; PUBLIC_RUN_ORIGIN?: string; }
}
declare module 'cloudflare:workers' { export const env: Cloudflare.Env; }
