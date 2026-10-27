declare namespace Cloudflare {
  interface Env { API_WORKER?: import('./lib/api-client').ServiceBinding; WEB_ORIGIN?: string; }
}
declare module 'cloudflare:workers' { export const env: Cloudflare.Env; }
