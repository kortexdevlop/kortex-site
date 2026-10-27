import {request as requestValidator,reply as replyValidator} from './generated-validators';
import protocol from '../../shared/schemas/protocol.schema.json' with {type:'json'};
export type ObjectValue=Record<string,unknown>;
export interface Request {v:1;request_id:string;kind:'init'|'render'|'action.invoke'|'ui.event'|'capability.result'|'state.restore'|'shutdown';payload:ObjectValue}
export interface HostError {code:string;message:string;retryable:boolean}
export interface Reply {v:1;request_id:string;ok:boolean;data?:ObjectValue;error?:HostError}
export interface CapabilityRequest {request_id:string;capability:string;payload:ObjectValue}
export const LIMIT=1048576, MEMORY_LIMIT=67108864, QUEUE_LIMIT=128;
export const uuid=(s:unknown):s is string=>typeof s==='string'&&/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i.test(s);
export const object=(v:unknown):v is ObjectValue=>v!==null&&typeof v==='object'&&!Array.isArray(v);
export function bounded(value:unknown):Uint8Array {const bytes=new TextEncoder().encode(JSON.stringify(value));if(bytes.length>LIMIT)throw Error('MESSAGE_LIMIT');return bytes;}
export function request(value:unknown):asserts value is Request {bounded(value);if(!requestValidator(value))throw Error('INVALID_PROTOCOL');}
export function reply(value:unknown):asserts value is Reply {bounded(value);if(!replyValidator(value))throw Error('INVALID_PROTOCOL');}
export const CAPABILITIES=['storage.read','storage.write','file.pick','file.read','file.save','network.fetch','clipboard.write'] as const;
export function capability(value:unknown):asserts value is CapabilityRequest {
 if(!object(value)||Object.keys(value).sort().join(',')!=='capability,payload,request_id'||!uuid(value.request_id)||!CAPABILITIES.includes(value.capability as any)||!object(value.payload))throw Error('INVALID_CAPABILITY');bounded(value);
}
export function error(code:string):HostError{return {code,message:code,retryable:['UNAVAILABLE','QUOTA','CANCELLED','OFFLINE'].includes(code)}}
