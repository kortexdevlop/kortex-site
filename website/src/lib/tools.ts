import {ToolSummary as checkSummary,ToolList as checkList,ToolDetail as checkDetail,ToolActions as checkActions} from './generated-validators.ts';
import type {Manifest, Contract} from '../../../shared/typescript/src/index.ts';
export interface ToolSummary {id:string;version:string;name:string;summary:string;category:string;tags:string[];icon_path:string;abi_version:'1.0';sha256:string;published_at:number;}
export interface ToolList {items:ToolSummary[];next_cursor:string|null;}
export interface ToolDetail {tool:ToolSummary;manifest:Manifest;contract:Contract;descriptor:Record<string,unknown>;}
export interface ToolActions {tool_id:string;version:string;actions:Contract['actions'];}
export const isToolSummary=(v:unknown):v is ToolSummary=>Boolean(checkSummary(v));
export const isToolList=(v:unknown):v is ToolList=>Boolean(checkList(v));
export const isToolDetail=(v:unknown):v is ToolDetail=>Boolean(checkDetail(v));
export const isToolActions=(v:unknown):v is ToolActions=>Boolean(checkActions(v));
export function palette(tool:ToolSummary):string {
  const names=['peach','lilac','mint','blue','yellow','pink','sand','aqua','sage'];
  let hash=0; for(const char of tool.category) hash=(hash*31+char.charCodeAt(0))>>>0;
  return names[hash%names.length];
}
export function toolHref(tool:ToolSummary):string {return `/tools/${encodeURIComponent(tool.id)}`;}
