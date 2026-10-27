import {defineConfig} from 'playwright/test';
// Chromium channel is configurable because Playwright browser downloads are optional on developer machines.
export default defineConfig({testDir:'.',testMatch:'five-tools.spec.ts',workers:1,timeout:60000,projects:[{name:'chromium',use:{browserName:'chromium',channel:process.env.KORTEX_CHROMIUM_CHANNEL||undefined}},{name:'webkit',use:{browserName:'webkit'}}],reporter:'list'});
