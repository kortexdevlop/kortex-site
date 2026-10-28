import {accountSession,accountRequest,libraryView,writeLibrary,type Preferences} from './account-client';
export async function connectWorkspace(){
 const owner=await accountSession(),preferences=await(await accountRequest('/api/v1/me/preferences')).json() as Preferences;if(!preferences.sync_enabled)return {enabled:false as const};let state=await libraryView();
 return {enabled:true as const,get items(){return state.items},preferences:preferences.preferences,async toggle(tool_id:string,enabled:boolean){const observed=state.items.find(item=>item.tool_id===tool_id)?.tags??[];if(enabled&&observed.length||!enabled&&!observed.length)return;state=await writeLibrary(owner,state,enabled?{adds:[{tool_id,tag:crypto.randomUUID()}],removes:[]}:{adds:[],removes:[{tool_id,tags:[...observed]}]})}};
}
