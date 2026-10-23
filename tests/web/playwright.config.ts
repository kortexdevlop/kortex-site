import {defineConfig} from 'playwright/test';
export default defineConfig({testDir:'../..',testMatch:['tests/web/**/*.spec.ts','web-adapter/tests/**/*.spec.ts'],workers:1,fullyParallel:false,timeout:60000,projects:[{name:'chromium',use:{browserName:'chromium'}},{name:'webkit',use:{browserName:'webkit'}}],reporter:'list'});
