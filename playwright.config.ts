import configuration from './tests/web/playwright.config';
const visual = process.argv.some(argument => argument.includes('tests/visual')) || process.env.KORTEX_DESIGN_TEST === '1';
// Worker processes re-evaluate this file without the parent CLI arguments.
if (visual) process.env.KORTEX_DESIGN_TEST = '1';
export default {
  ...configuration,
  testDir: '.',
  testMatch: visual ? ['tests/visual/**/*.spec.ts'] : configuration.testMatch,
  use: {...configuration.use, ...(visual ? {baseURL:'http://127.0.0.1:4321'} : {})},
  projects: configuration.projects?.map(project => ({...project, use: {...project.use, ...(visual ? {baseURL:'http://127.0.0.1:4321'} : {})}})),
  reporter: visual ? [['list'], ['json', {outputFile:'docs/evidence/design10-playwright.json'}]] : configuration.reporter,
  ...(visual ? {webServer: {
    command: 'pnpm --filter @kortex/website dev',
    url: 'http://127.0.0.1:4321/__dev/design',
    reuseExistingServer: false,
    timeout: 120000,
  }} : {}),
};
