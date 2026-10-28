import {test} from 'playwright/test';
import {accountScenario} from './account-scenario.mjs';
test('compte réel : invité, deux appareils, conflits, favoris, export et purge',async({browser,baseURL})=>{
 if(!process.env.KORTEX_ACCOUNT_EMAIL||!process.env.KORTEX_ACCOUNT_PASSWORD)throw Error('Run scripts/test-account-staging.mjs to provision and clean the isolated fixture');
 await accountScenario(browser,baseURL!,process.env.KORTEX_ACCOUNT_EMAIL,process.env.KORTEX_ACCOUNT_PASSWORD);
});
