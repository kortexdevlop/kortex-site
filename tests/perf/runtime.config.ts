import {defineConfig} from 'playwright/test';
export default defineConfig({testDir:'.',testMatch:'runtime-web.spec.ts',workers:1,timeout:240000,expect:{timeout:45000},use:{browserName:'chromium'},reporter:[['list'],['json',{outputFile:process.cwd()+'/target/ci-reports/runtime-web-playwright.json'}]]});
