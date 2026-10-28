import {defineConfig} from 'playwright/test';
export default defineConfig({testDir:'./e2e',testMatch:'workflow-editor.spec.ts',workers:1,timeout:120000,expect:{timeout:30000},use:{baseURL:'https://kortex-web-staging.userpgl.workers.dev',browserName:'chromium',trace:'off',video:'off',screenshot:'off'},reporter:'list'});
