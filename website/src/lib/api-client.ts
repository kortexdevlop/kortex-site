import {isToolList,isToolDetail,isToolActions,type ToolList,type ToolDetail,type ToolActions} from './tools.ts';
export interface ServiceBinding {fetch(request:Request):Promise<Response>;}
export class ApiUnavailable extends Error {status:number;constructor(status=503){super('Catalogue indisponible');this.status=status;}}
export class ApiClient {
  private binding?:ServiceBinding;private browserFetch:typeof fetch;
  constructor(binding?:ServiceBinding,browserFetch:typeof fetch=fetch) {this.binding=binding;this.browserFetch=browserFetch;}
  private async get<T>(path:string, validate:(value:unknown)=>value is T):Promise<T> {
    const controller=new AbortController();
    let reader:ReadableStreamDefaultReader<Uint8Array>|undefined;
    let timer:ReturnType<typeof setTimeout>;
    const expired=new Promise<never>((_,reject)=>{timer=setTimeout(()=>{controller.abort();reject(new ApiUnavailable());},4000);});
    try {
      const response=await Promise.race([this.binding
        ? this.binding.fetch(new Request(`https://kortex-api.internal${path}`,{signal:controller.signal,redirect:'manual',headers:{Accept:'application/json'}}))
        : typeof window!=='undefined' ? this.browserFetch(path,{credentials:'same-origin',signal:controller.signal,redirect:'error'}) : Promise.resolve(null),expired]);
      if(!response?.ok) throw new ApiUnavailable(response?.status===404?404:503);
      if(!response.headers.get('content-type')?.includes('application/json'))throw new ApiUnavailable();
      reader=response.body?.getReader();if(!reader)throw new ApiUnavailable();
      const parts:Uint8Array[]=[];let size=0;
      while(true){const {done,value}=await Promise.race([reader.read(),expired]);if(done)break;size+=value.length;if(size>1048576){void reader.cancel();throw new ApiUnavailable();}parts.push(value);}
      const bytes=new Uint8Array(size);let offset=0;for(const part of parts){bytes.set(part,offset);offset+=part.length;}
      const value:unknown=JSON.parse(new TextDecoder().decode(bytes));if(!validate(value))throw new ApiUnavailable();return value;
    }catch(error){throw error instanceof ApiUnavailable?error:new ApiUnavailable();}finally{clearTimeout(timer!);void reader?.cancel().catch(()=>{});}
  }
  listTools(query:URLSearchParams=new URLSearchParams()):Promise<ToolList>{return this.get(`/api/v1/tools${query.size?'?'+query:''}`,isToolList);}
  async getTool(id:string):Promise<ToolDetail>{const detail=await this.get(`/api/v1/tools/${encodeURIComponent(id)}`,isToolDetail);const {tool,manifest,contract,descriptor}=detail;if(tool.id!==id||manifest.id!==id||contract.tool_id!==id||descriptor.tool_id!==id||manifest.version!==tool.version||descriptor.version!==tool.version||descriptor.sha256!==tool.sha256)throw new ApiUnavailable();return detail;}
  getActions(id:string):Promise<ToolActions>{return this.get(`/api/v1/tools/${encodeURIComponent(id)}/actions`,isToolActions);}
}
