import {defineConfig} from 'playwright/test';
export default defineConfig({testDir:'.',testMatch:'phase12-csv-staging.spec.ts',workers:1,timeout:120000,projects:[{name:'chromium',use:{browserName:'chromium',channel:process.env.KORTEX_CHROMIUM_CHANNEL||undefined}}],reporter:[['list'],['json',{outputFile:process.cwd()+'/docs/evidence/phase12-csv-staging-playwright.json'}]]});
