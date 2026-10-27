import configuration from './website.config';
export default {...configuration,testMatch:['tests/seo/sitemap.spec.ts','tests/e2e/search.spec.ts'],reporter:[['list'],['json',{outputFile:process.cwd()+'/docs/evidence/website-11.4-11.6-playwright.json'}]]};
