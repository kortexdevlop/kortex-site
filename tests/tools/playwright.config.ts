import {defineConfig} from 'playwright/test';
export default defineConfig({testDir:'.',testMatch:'phase12.spec.ts',workers:1,timeout:60000,projects:[{name:'chromium',use:{browserName:'chromium'}},{name:'webkit',use:{browserName:'webkit'}}],reporter:[['list'],['json',{outputFile:process.cwd()+'/docs/evidence/phase12-local-playwright.json'}]]});
