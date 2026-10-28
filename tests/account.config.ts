import {defineConfig} from 'playwright/test';
export default defineConfig({testDir:'./e2e',testMatch:'account.spec.ts',workers:1,timeout:180000,expect:{timeout:30000},use:{baseURL:'https://kortex-web-staging.userpgl.workers.dev',trace:'off',screenshot:'off',video:'off'},projects:[{name:'chromium-windows',use:{browserName:'chromium'}}],reporter:'list'});
