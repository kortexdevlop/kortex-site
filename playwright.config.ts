import nextConfiguration from './tests/website-next.config';
import configuration from './tests/web/playwright.config';
import websiteConfiguration from './tests/website.config';
const website=process.argv.some(argument=>/tests[\\/]e2e[\\/](?:catalog|open-tool)\.spec|tests[\\/]visual[\\/]home\.spec/.test(argument))||process.env.KORTEX_WEBSITE_TEST==='1';
if(website)process.env.KORTEX_WEBSITE_TEST='1';
const visual = process.argv.some(argument => /tests[\\/]visual|tests[\\/]design[\\/].*\.spec/.test(argument)) || process.env.KORTEX_DESIGN_TEST === '1';
// Worker processes re-evaluate this file without the parent CLI arguments.
if (visual) process.env.KORTEX_DESIGN_TEST = '1';
const nextWebsite=process.argv.some(argument=>/tests[\\/](?:seo[\\/]sitemap|e2e[\\/]search)\.spec/.test(argument))||process.env.KORTEX_WEBSITE_NEXT==='1';if(nextWebsite)process.env.KORTEX_WEBSITE_NEXT='1';
export default nextWebsite ? nextConfiguration : website ? websiteConfiguration : {
  ...configuration,
  testDir: '.',
  testIgnore:['tests/e2e/search.spec.ts','tests/integration/hot-publish.spec.ts','tests/visual/home.spec.ts','tests/e2e/catalog.spec.ts','tests/e2e/open-tool.spec.ts'],
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
