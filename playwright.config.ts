import configuration from './tests/web/playwright.config';
const visual = process.argv.some(argument => /tests[\\/]visual|tests[\\/]design[\\/].*\.spec/.test(argument)) || process.env.KORTEX_DESIGN_TEST === '1';
// Worker processes re-evaluate this file without the parent CLI arguments.
if (visual) process.env.KORTEX_DESIGN_TEST = '1';
export default {
  ...configuration,
  testDir: '.',
  testMatch: visual ? ['tests/visual/**/*.spec.ts','tests/design/**/*.spec.ts'] : configuration.testMatch,
  use: {...configuration.use, ...(visual ? {baseURL:'http://127.0.0.1:4321',deviceScaleFactor:1} : {})},
  projects: configuration.projects?.map(project => ({...project, use: {...project.use, ...(visual ? {baseURL:'http://127.0.0.1:4321',...(project.name==='chromium'?{launchOptions:{ignoreDefaultArgs:['--hide-scrollbars']}}:{})} : {})}})),
  reporter: visual ? [['list'], ['json', {outputFile:process.env.KORTEX_DESIGN_REPORT||'docs/evidence/design-library-playwright.json'}]] : configuration.reporter,
  ...(visual ? {webServer: {
    command: 'pnpm --filter @kortex/website dev',
    url: 'http://127.0.0.1:4321/__dev/design',
    reuseExistingServer: process.env.KORTEX_DESIGN_REUSE==='1',
    timeout: 120000,
  }} : {}),
};
