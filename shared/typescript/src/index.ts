import { validate } from './validate.ts';
export type { Result, Violation } from './validate.ts';
export type Permission = 'storage.read'|'storage.write'|'file.pick'|'file.read'|'file.save'|'network.fetch'|'clipboard.write';
export interface Manifest { format_version:'1.0'; id:string; version:string; name:string; summary:string; description:string; category:string; tags:string[]; seo:{title:string;description:string}; icon_path:string; abi_version:'1.0';ui_schema_version:'1.0';permissions:Permission[];entrypoint:'module.wasm';min_runtime:'1.0.0' }
export interface ValueSchema { type:'object'|'array'|'string'|'number'|'integer'|'boolean'|'null';properties?:Record<string,ValueSchema>;required?:string[];additionalProperties?:boolean;items?:ValueSchema;enum?:Array<string|number|boolean|null>;minimum?:number;maximum?:number;minLength?:number;maxLength?:number;minItems?:number;maxItems?:number }
export interface Action { id:string;description:string;input_schema:ValueSchema;output_schema:ValueSchema;permissions:Permission[];side_effect:'none'|'local-write'|'network';idempotent:boolean;timeout_ms:number }
export interface Contract { contract_version:'1.0';tool_id:string;actions:Action[] }
export type UiType='column'|'row'|'grid'|'card'|'text'|'heading'|'button'|'input'|'textarea'|'select'|'checkbox'|'table'|'progress'|'alert'|'svg-chart'|'toolbar'|'tabs';
export interface UiNode {type:UiType;key:string;props:Record<string,unknown>;children:UiNode[]}
export interface Workflow { workflow_version:'1.0';id:string;name:string;nodes:Array<{id:string;action_ref:string;input_bindings:Record<string,{value:unknown}|{from_node:string;source_pointer:string}>;requires_confirmation:boolean}>;edges:Array<{from:string;to:string}> }
export const validateManifest=(value:unknown)=>validate<Manifest>('manifest',value);
export const validateContract=(value:unknown)=>validate<Contract>('contract',value);
export const validateUi=(value:unknown)=>validate<UiNode>('ui',value);
export const validateWorkflow=(value:unknown)=>validate<Workflow>('workflow',value);
