import {defineConfig} from 'playwright/test';
export default defineConfig({testDir:'.',testMatch:['e2e/ai.spec.ts','security/ai-privacy.spec.ts'],workers:1,timeout:180000,expect:{timeout:45000},use:{baseURL:'https://kortex-web-staging.userpgl.workers.dev',browserName:'chromium',trace:'off',screenshot:'off',video:'off'},reporter:'list'});
